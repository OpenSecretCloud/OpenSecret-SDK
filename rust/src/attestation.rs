use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ring::{digest, signature};
use serde::{Deserialize, Serialize};
use serde_cbor::Value as CborValue;
use sha2::{Digest, Sha384};
use x509_parser::prelude::*;
use crate::error::{Error, Result};

// AWS Nitro Root Certificate (production)
const AWS_NITRO_ROOT_CERT: &[u8] = include_bytes!("../assets/aws_nitro_root.der");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationDocument {
    pub module_id: String,
    pub timestamp: u64,
    pub digest: String,
    pub pcrs: std::collections::HashMap<usize, Vec<u8>>,
    pub certificate: Vec<u8>,
    pub cabundle: Vec<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
    pub user_data: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
}

pub struct AttestationVerifier {
    expected_pcrs: Option<std::collections::HashMap<usize, Vec<u8>>>,
    allow_debug: bool,
}

impl AttestationVerifier {
    pub fn new() -> Self {
        Self {
            expected_pcrs: None,
            allow_debug: cfg!(feature = "mock-attestation"),
        }
    }

    pub fn with_expected_pcrs(mut self, pcrs: std::collections::HashMap<usize, Vec<u8>>) -> Self {
        self.expected_pcrs = Some(pcrs);
        self
    }

    pub fn verify_attestation_document(
        &self,
        document_b64: &str,
        expected_nonce: &str,
    ) -> Result<AttestationDocument> {
        let document_bytes = BASE64.decode(document_b64)?;
        
        // Parse COSE_Sign1 structure
        let cbor_value: CborValue = serde_cbor::from_slice(&document_bytes)
            .map_err(|e| Error::Cbor(e))?;
        
        let cose_sign1 = match &cbor_value {
            CborValue::Array(arr) => arr,
            _ => return Err(Error::AttestationVerificationFailed("Invalid COSE_Sign1 structure".to_string())),
        };
        
        if cose_sign1.len() != 4 {
            return Err(Error::AttestationVerificationFailed(
                "COSE_Sign1 must have 4 elements".to_string(),
            ));
        }
        
        // Extract components
        let protected = match &cose_sign1[0] {
            CborValue::Bytes(b) => b,
            _ => return Err(Error::AttestationVerificationFailed("Invalid protected header".to_string())),
        };
        
        let payload = match &cose_sign1[2] {
            CborValue::Bytes(b) => b,
            _ => return Err(Error::AttestationVerificationFailed("Invalid payload".to_string())),
        };
        
        let signature = match &cose_sign1[3] {
            CborValue::Bytes(b) => b,
            _ => return Err(Error::AttestationVerificationFailed("Invalid signature".to_string())),
        };
        
        // Parse attestation document from payload
        let doc_cbor: CborValue = serde_cbor::from_slice(payload)
            .map_err(|e| Error::Cbor(e))?;
        
        let doc = self.parse_attestation_document(&doc_cbor)?;
        
        // Verify nonce
        if let Some(nonce_bytes) = &doc.nonce {
            let nonce_str = String::from_utf8(nonce_bytes.clone())
                .map_err(|e| Error::AttestationVerificationFailed(format!("Invalid nonce encoding: {}", e)))?;
            
            if nonce_str != expected_nonce {
                return Err(Error::AttestationVerificationFailed(
                    "Nonce mismatch".to_string(),
                ));
            }
        } else {
            return Err(Error::AttestationVerificationFailed(
                "Missing nonce in attestation document".to_string(),
            ));
        }
        
        // Verify certificate chain
        self.verify_certificate_chain(&doc)?;
        
        // Verify signature
        self.verify_signature(protected, payload, signature, &doc)?;
        
        // Verify PCRs if expected
        if let Some(expected_pcrs) = &self.expected_pcrs {
            self.verify_pcrs(&doc, expected_pcrs)?;
        }
        
        Ok(doc)
    }
    
    fn parse_attestation_document(&self, cbor: &CborValue) -> Result<AttestationDocument> {
        let map = match cbor {
            CborValue::Map(m) => m,
            _ => return Err(Error::AttestationVerificationFailed("Invalid attestation document format".to_string())),
        };
        
        let mut doc = AttestationDocument {
            module_id: String::new(),
            timestamp: 0,
            digest: String::new(),
            pcrs: std::collections::HashMap::new(),
            certificate: Vec::new(),
            cabundle: Vec::new(),
            public_key: None,
            user_data: None,
            nonce: None,
        };
        
        for (key, value) in map {
            let key_str = match key {
                CborValue::Text(s) => s.as_str(),
                _ => return Err(Error::AttestationVerificationFailed("Invalid key in attestation document".to_string())),
            };
            
            match key_str {
                "module_id" => {
                    doc.module_id = match value {
                        CborValue::Text(s) => s.clone(),
                        _ => return Err(Error::AttestationVerificationFailed("Invalid module_id".to_string())),
                    };
                }
                "timestamp" => {
                    doc.timestamp = match value {
                        CborValue::Integer(i) => *i as u64,
                        _ => return Err(Error::AttestationVerificationFailed("Invalid timestamp".to_string())),
                    };
                }
                "digest" => {
                    doc.digest = match value {
                        CborValue::Text(s) => s.clone(),
                        _ => return Err(Error::AttestationVerificationFailed("Invalid digest".to_string())),
                    };
                }
                "pcrs" => {
                    let pcrs_map = match value {
                        CborValue::Map(m) => m,
                        _ => return Err(Error::AttestationVerificationFailed("Invalid PCRs format".to_string())),
                    };
                    
                    for (pcr_key, pcr_value) in pcrs_map {
                        let index = match pcr_key {
                            CborValue::Integer(i) => *i as usize,
                            _ => return Err(Error::AttestationVerificationFailed("Invalid PCR index".to_string())),
                        };
                        
                        let pcr_bytes = match pcr_value {
                            CborValue::Bytes(b) => b.clone(),
                            _ => return Err(Error::AttestationVerificationFailed("Invalid PCR value".to_string())),
                        };
                        
                        doc.pcrs.insert(index, pcr_bytes);
                    }
                }
                "certificate" => {
                    doc.certificate = match value {
                        CborValue::Bytes(b) => b.clone(),
                        _ => return Err(Error::AttestationVerificationFailed("Invalid certificate".to_string())),
                    };
                }
                "cabundle" => {
                    let bundle = match value {
                        CborValue::Array(a) => a,
                        _ => return Err(Error::AttestationVerificationFailed("Invalid cabundle".to_string())),
                    };
                    
                    for cert in bundle {
                        let cert_bytes = match cert {
                            CborValue::Bytes(b) => b.clone(),
                            _ => return Err(Error::AttestationVerificationFailed("Invalid certificate in bundle".to_string())),
                        };
                        doc.cabundle.push(cert_bytes);
                    }
                }
                "public_key" => {
                    doc.public_key = match value {
                        CborValue::Bytes(b) => Some(b.clone()),
                        _ => None,
                    };
                }
                "user_data" => {
                    doc.user_data = match value {
                        CborValue::Bytes(b) => Some(b.clone()),
                        _ => None,
                    };
                }
                "nonce" => {
                    doc.nonce = match value {
                        CborValue::Bytes(b) => Some(b.clone()),
                        _ => None,
                    };
                }
                _ => {} // Ignore unknown fields
            }
        }
        
        Ok(doc)
    }
    
    fn verify_certificate_chain(&self, doc: &AttestationDocument) -> Result<()> {
        // In mock mode, skip certificate verification
        if self.allow_debug && doc.module_id.starts_with("mock-") {
            return Ok(());
        }
        
        // Parse the AWS root certificate
        let (_, _aws_root_cert) = X509Certificate::from_der(AWS_NITRO_ROOT_CERT)
            .map_err(|e| Error::AttestationVerificationFailed(format!("Failed to parse AWS root certificate: {:?}", e)))?;
        
        // ⚠️ ⚠️ ⚠️ WARNING: INCOMPLETE CERTIFICATE CHAIN VERIFICATION ⚠️ ⚠️ ⚠️
        // 
        // This implementation currently only:
        // 1. Checks that certificates are not expired
        // 2. Verifies the COSE_Sign1 signature (which IS properly implemented)
        // 
        // It does NOT:
        // 1. Verify that each certificate is signed by the next one in the chain
        // 2. Verify that the chain ultimately leads back to the AWS Nitro root certificate
        // 3. Validate certificate constraints, key usage, or other X.509 extensions
        // 
        // This means that while we verify the attestation document signature is valid,
        // we don't fully verify the certificate chain's authenticity. A malicious actor
        // could potentially create their own certificate chain with valid signatures.
        // 
        // For production use, implement full X.509 chain validation using a proper
        // PKI library like webpki or similar.
        // ⚠️ ⚠️ ⚠️ END WARNING ⚠️ ⚠️ ⚠️
        
        // Verify each certificate in the chain
        let mut prev_cert: Option<X509Certificate> = None;
        
        for (i, cert_der) in doc.cabundle.iter().enumerate() {
            let (_, cert) = X509Certificate::from_der(cert_der)
                .map_err(|e| Error::AttestationVerificationFailed(format!("Failed to parse certificate {}: {:?}", i, e)))?;
            
            // Check certificate validity
            if !cert.validity().is_valid() {
                return Err(Error::AttestationVerificationFailed(
                    format!("Certificate {} is expired", i),
                ));
            }
            
            prev_cert = Some(cert);
        }
        
        if prev_cert.is_none() {
            return Err(Error::AttestationVerificationFailed(
                "Certificate chain is empty".to_string()
            ));
        }
        
        // The leaf certificate should also be valid
        let (_, leaf_cert) = X509Certificate::from_der(&doc.certificate)
            .map_err(|e| Error::AttestationVerificationFailed(format!("Failed to parse leaf certificate: {:?}", e)))?;
        
        if !leaf_cert.validity().is_valid() {
            return Err(Error::AttestationVerificationFailed(
                "Leaf certificate is expired".to_string(),
            ));
        }
        
        Ok(())
    }
    
    fn verify_signature(
        &self,
        protected: &[u8],
        payload: &[u8],
        signature_bytes: &[u8],
        doc: &AttestationDocument,
    ) -> Result<()> {
        // In mock mode, skip signature verification
        if self.allow_debug && doc.module_id.starts_with("mock-") {
            return Ok(());
        }
        
        
        // Parse the leaf certificate
        let (_, cert) = X509Certificate::from_der(&doc.certificate)
            .map_err(|e| Error::AttestationVerificationFailed(format!("Failed to parse leaf certificate: {:?}", e)))?;
        
        // Extract the public key bytes from the certificate
        let public_key_info = cert.public_key();
        let full_public_key = public_key_info.raw;
        
        // For EC keys, the actual point is at the end after the algorithm parameters
        // For P-384, the uncompressed point is 97 bytes (0x04 + 48 bytes X + 48 bytes Y)
        let public_key_bytes = if full_public_key.len() > 97 {
            // Skip the ASN.1 structure and get just the EC point
            &full_public_key[full_public_key.len() - 97..]
        } else {
            full_public_key
        };
        
        // Create the COSE_Sign1 signature structure
        // This follows the COSE specification for the data to be signed
        let sig_structure = create_sig_structure(protected, payload);
        
        // For ECDSA P-384 with SHA-384 (which is what AWS Nitro uses)
        // AWS Nitro uses raw signatures (r||s), not ASN.1 encoded
        let public_key = signature::UnparsedPublicKey::new(
            &signature::ECDSA_P384_SHA384_FIXED,
            public_key_bytes
        );
        
        // Verify the signature
        public_key.verify(&sig_structure, signature_bytes)
            .map_err(|_| Error::AttestationVerificationFailed(
                "Signature verification failed".to_string()
            ))?;
        
        Ok(())
    }
    
    fn verify_pcrs(
        &self,
        doc: &AttestationDocument,
        expected: &std::collections::HashMap<usize, Vec<u8>>,
    ) -> Result<()> {
        for (index, expected_value) in expected {
            match doc.pcrs.get(index) {
                Some(actual_value) => {
                    if actual_value != expected_value {
                        return Err(Error::AttestationVerificationFailed(
                            format!("PCR{} mismatch", index),
                        ));
                    }
                }
                None => {
                    return Err(Error::AttestationVerificationFailed(
                        format!("PCR{} missing", index),
                    ));
                }
            }
        }
        Ok(())
    }
}

fn create_sig_structure(protected: &[u8], payload: &[u8]) -> Vec<u8> {
    // Create the COSE_Sign1 signature structure as a CBOR array
    // ["Signature1", protected, external_aad, payload]
    let sig_structure = CborValue::Array(vec![
        CborValue::Text("Signature1".to_string()),
        CborValue::Bytes(protected.to_vec()),
        CborValue::Bytes(vec![]), // empty external AAD
        CborValue::Bytes(payload.to_vec()),
    ]);
    
    // Encode to CBOR bytes
    serde_cbor::to_vec(&sig_structure)
        .expect("Failed to encode signature structure")
}

#[cfg(feature = "mock-attestation")]
pub fn create_mock_attestation_document(nonce: &str) -> Result<String> {
    use serde_cbor::to_vec;
    use std::collections::HashMap;
    
    let mut pcrs = HashMap::new();
    pcrs.insert(0, vec![0u8; 48]); // Mock PCR0
    
    let doc = AttestationDocument {
        module_id: "mock-module".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
        digest: "SHA384".to_string(),
        pcrs,
        certificate: vec![0u8; 256], // Mock certificate
        cabundle: vec![vec![0u8; 256]], // Mock CA bundle
        public_key: None,
        user_data: None,
        nonce: Some(nonce.as_bytes().to_vec()),
    };
    
    // Create a mock COSE_Sign1 structure
    let payload = to_vec(&doc).map_err(|e| Error::Cbor(e))?;
    let protected = vec![0u8; 32]; // Mock protected header
    let signature = vec![0u8; 64]; // Mock signature
    
    let cose_sign1 = vec![
        CborValue::Bytes(protected),
        CborValue::Map(std::collections::BTreeMap::new()), // Empty unprotected headers
        CborValue::Bytes(payload),
        CborValue::Bytes(signature),
    ];
    
    let cose_bytes = to_vec(&CborValue::Array(cose_sign1)).map_err(|e| Error::Cbor(e))?;
    Ok(BASE64.encode(cose_bytes))
}