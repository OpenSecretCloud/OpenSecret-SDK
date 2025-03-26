/**
 * Valid PCR0 values for production environments
 */
const DEFAULT_PCR0_VALUES = [
  "eeddbb58f57c38894d6d5af5e575fbe791c5bf3bbcfb5df8da8cfcf0c2e1da1913108e6a762112444740b88c163d7f4b",
  "74ed417f88cb0ca76c4a3d10f278bd010f1d3f95eafb254d4732511bb50e404507a4049b779c5230137e4091a5582271",
  "9043fcab93b972d3c14ad2dc8fa78ca7ad374fc937c02435681772a003f7a72876bc4d578089b5c4cf3fe9b480f1aabb",
  "52c3595b151d93d8b159c257301bfd5aa6f49210de0c55a6cd6df5ebeee44e4206cab950500f5d188f7fa14e6d900b75",
  "91cb67311e910cce68cd5b7d0de77aa40610d87c6681439b44c46c3ff786ae643956ab2c812478a1da8745b259f07a45",
  "859065ac81b81d3735130ba08b8af72a7256b603fefb74faabae25ed28cca6edcaa7c10ea32b5948d675c18a9b0f2b1d",
  "acd82a7d3943e23e95a9dc3ce0b0107ea358d6287f9e3afa245622f7c7e3e0a66142a928b6efcc02f594a95366d3a99d"
];

/**
 * Valid PCR0 values for development environments
 */
const DEFAULT_PCR0_VALUES_DEV = [
  "62c0407056217a4c10764ed9045694c29fa93255d3cc04c2f989cdd9a1f8050c8b169714c71f1118ebce2fcc9951d1a9",
  "cb95519905443f9f66f05f63c548b61ad1561a27fd5717b69285861aaea3c3063fe12a2571773b67fea3c6c11b4d8ec6",
  "deb5895831b5e4286f5a2dcf5e9c27383821446f8df2b465f141d10743599be20ba3bb381ce063bf7139cc89f7f61d4c",
  "70ba26c6af1ec3b57ce80e1adcc0ee96d70224d4c7a078f427895cdf68e1c30f09b5ac4c456588d872f3f21ff77c036b",
  "669404ea71435b8f498b48db7816a5c2ab1d258b1a77685b11d84d15a73189504d79c4dee13a658de9f4a0cbfc39cfe8",
  "a791bf92c25ffdfd372660e460a0e238c6778c090672df6509ae4bc065cf8668b6baac6b6a11d554af53ee0ff0172ad5",
  "c4285443b87b9b12a6cea3bef1064ec060f652b235a297095975af8f134e5ed65f92d70d4616fdec80af9dff48bb9f35"
];

import { validatePcrAgainstHistory } from "./pcrHistory";

export type Pcr0ValidationResult = {
  isMatch: boolean;
  text: string;
};

export type PcrConfig = {
  pcr0Values?: string[];
  pcr0DevValues?: string[];
  remoteValidationUrl?: string;
};

/**
 * Validates a PCR0 hash and returns information about the match
 * @param hash - The PCR0 hash to validate
 * @param config - Optional configuration with custom PCR0 values
 * @returns Object containing match status and descriptive text
 */
export async function validatePcr0Hash(
  hash: string,
  config?: PcrConfig
): Promise<Pcr0ValidationResult> {
  const validPcr0Values = [...(config?.pcr0Values || []), ...DEFAULT_PCR0_VALUES];
  const validPcr0DevValues = [...(config?.pcr0DevValues || []), ...DEFAULT_PCR0_VALUES_DEV];

  // First try local validation with hardcoded values
  if (validPcr0Values.includes(hash)) {
    return {
      isMatch: true,
      text: "PCR0 matches a known good value"
    };
  }

  if (validPcr0DevValues.includes(hash)) {
    return {
      isMatch: true,
      text: "PCR0 matches development enclave"
    };
  }

  // Check remote validation if URL provided
  if (config?.remoteValidationUrl) {
    try {
      const pcrSet = { PCR0: hash, PCR1: "", PCR2: "" };
      const remoteValidation = await validatePcrAgainstHistory(pcrSet, config);
      return remoteValidation;
    } catch (error) {
      console.error("Error with remote PCR validation:", error);
      // Continue with default validation path (do not return error from remote validation)
    }
  }

  return {
    isMatch: false,
    text: "PCR0 does not match a known good value"
  };
}
