import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

// import Feature1ImageUrl from '@site/static/img/dimo_feature_declaritive_manifests.webp'

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  WebP: string;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Declarative Manifests',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    WebP: require('@site/static/img/dimo_feature_declaritive_manifests.webp').default,
    description: (
      <>
        Stop memorizing complex CLI flags. Define your containers as code using 
        simple <code>.ini</code> files and share them with your team.
      </>
    ),
  },
  {
    title: 'Interactive Wizard',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    WebP: require('@site/static/img/dimo_feature_interactive_wizard.webp').default,
    description: (
      <>
        New to Distrobox? Use the built-in TUI wizard (<code>dimo create</code>) 
        to visually build complex container configurations step-by-step.
      </>
    ),
  },
  {
    title: 'The Cauldron',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    WebP: require('@site/static/img/dimo_feature_cauldron.png').default,
    description: (
      <>
        Manage your entire container ecosystem with a modern terminal interface. 
        Monitor status, manage homes, and clean up storage effortlessly.
      </>
    ),
  },
];

function Feature({title, Svg, WebP, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        {/* <Svg className={styles.featureSvg} role="img" /> */}
        <img src={WebP}></img>
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
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
