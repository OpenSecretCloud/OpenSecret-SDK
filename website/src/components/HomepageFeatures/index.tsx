import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import SecuritySvg from '@site/static/img/branding-fixed/security.svg';
import UxSvg from '@site/static/img/branding-fixed/ux.svg';
import AiSvg from '@site/static/img/branding-fixed/ai.svg';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Secure by Design',
    Svg: SecuritySvg,
    description: (
      <>
        OpenSecret SDK is built with security as the core principle, leveraging
        remote attestation, client-side encryption, and secure authentication.
      </>
    ),
  },
  {
    title: 'React Integration',
    Svg: UxSvg,
    description: (
      <>
        Seamlessly integrate with your React applications using our purpose-built
        hooks and components for state management and authentication.
      </>
    ),
  },
  {
    title: 'Privacy-Preserving AI',
    Svg: AiSvg,
    description: (
      <>
        Leverage AI capabilities while maintaining data privacy through our secure
        APIs and encryption protocols. Your data remains protected at all times.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}