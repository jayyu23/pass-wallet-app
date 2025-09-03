import { ConnectButton } from '@rainbow-me/rainbowkit';
import type { NextPage } from 'next';
import Head from 'next/head';
import styles from '../styles/Home.module.css';
import { useAccount } from 'wagmi';
import Navbar from '../components/Navbar';
import AccountList from '../components/AccountList';

const Home: NextPage = () => {
  const { address, isConnected } = useAccount();

  const contributions = [
    {
      title: "Provenance-Based Access Control",
      description: "Novel access control mechanism where authority over assets is determined by their origin and transfer history, rather than traditional role-based approaches.",
      icon: "🔗"
    },
    {
      title: "Inbox-Outbox Mechanism",
      description: "Custodial system that mediates secret key access through a secure channel, enabling multi-entity control without compromising security.",
      icon: "📥"
    },
    {
      title: "Privacy-Preserving Transfers",
      description: "Internal transfers between subaccounts leave no on-chain trace, making multi-user wallets indistinguishable from regular accounts.",
      icon: "🔒"
    },
    {
      title: "Formal Verification",
      description: "Key security properties including privacy and policy integrity are formally verified using Lean 4 theorem prover.",
      icon: "✅"
    },
    {
      title: "TEE-Encumbered Keys",
      description: "Private keys are managed within Trusted Execution Environments, ensuring secure execution isolated from the host system.",
      icon: "🛡️"
    },
    {
      title: "No Smart Contract Dependency",
      description: "Demonstrates feasibility without requiring smart contract deployments, working directly with EOA addresses.",
      icon: "⚡"
    }
  ];

  return (
    <div className={styles.container}>
      <Head>
        <title>PASS - Provenanced Access Subaccount System | Academic Prototype</title>
        <meta name="description" content="Academic prototype of the Provenanced Access Subaccount System (PASS), a novel wallet architecture enabling multiple entities to securely share control of a single blockchain address with formal verification." />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Navbar />

      <main className={styles.main}>
        {!isConnected ? (
          <>
            {/* Hero Section */}
            <div className={styles.hero}>
              <div className={styles.heroContent}>
                <h1 className={styles.heroTitle}>
                  Provenanced Access Subaccount System (PASS)
                  <span className={styles.subtitle}>Academic Research Prototype</span>
                </h1>
                <p className={styles.heroDescription}>
                  A novel wallet architecture enabling multiple entities to securely share control of a single blockchain address through provenance-based access control and TEE-encumbered secret key management.
                </p>
                <div className={styles.connectContainer}>
                  <ConnectButton />
                  <p className={styles.connectHint}>Connect your wallet to explore the PASS prototype</p>
                </div>
              </div>
            </div>

            {/* Research Abstract Section */}
            <section className={styles.abstractSection}>
              <h2 className={styles.sectionTitle}>Research Abstract</h2>
              <div className={styles.abstractContent}>
                <p>
                  Blockchain wallets traditionally operate under a single-entity ownership model where possession of a private key grants complete control over all assets. This paradigm becomes limiting as blockchain applications evolve toward more complex access patterns, such as with Trusted Execution Environment (TEE) based private key encumbrance.
                </p>
                <p>
                  We present the Provenanced Access Subaccount System (PASS), a novel wallet architecture that enables multiple entities to securely share control of a single blockchain address. Unlike existing approaches that rely on role-based or attribute-based access control, PASS introduces provenance-based access control, where authority over assets is determined by their origin and transfer history, and secret key access is mediated by a custodial Inbox-Outbox mechanism.
                </p>
                <p>
                  Our design allows multi-entity control of a single wallet address with a strong notion of privacy, where internal transfers between subaccounts leave no on-chain trace, and a multi-user PASS wallet is indistinguishable from a regular user account. We outline the core design of PASS and formally verify key security properties, such as privacy and policy integrity, with Lean 4. We also implement a prototype in TypeScript with WalletConnect integration for Ethereum Virtual Machine (EVM) blockchains and with a TEE-encumbered secret key, demonstrating PASS's feasibility without any smart contract deployments.
                </p>
                <p>
                  This work advances wallet security by combining the flexibility of multi-user access models with strong privacy guarantees and a formal verification approach.
                </p>
              </div>
            </section>

            {/* Research Contributions Section */}
            <section className={styles.featuresSection}>
              <h2 className={styles.sectionTitle}>Research Contributions</h2>
              <div className={styles.featuresGrid}>
                {contributions.map((contribution, index) => (
                  <div key={index} className={styles.featureCard}>
                    <div className={styles.featureIcon}>{contribution.icon}</div>
                    <h3 className={styles.featureTitle}>{contribution.title}</h3>
                    <p className={styles.featureDescription}>{contribution.description}</p>
                  </div>
                ))}
              </div>
            </section>

            {/* Formal Verification Section */}
            <section className={styles.securitySection}>
              <div className={styles.securityContent}>
                <h2 className={styles.sectionTitle}>Formal Verification & Privacy Guarantees</h2>
                <div className={styles.securityGrid}>
                  <div className={styles.securityItem}>
                    <h3>🔍 Privacy Property Verification</h3>
                    <p>Formally verified that multi-user PASS wallets are indistinguishable from regular user accounts on-chain</p>
                  </div>
                  <div className={styles.securityItem}>
                    <h3>✅ Policy Integrity Verification</h3>
                    <p>Proven that access control policies cannot be violated through formal verification with Lean 4</p>
                  </div>
                  <div className={styles.securityItem}>
                    <h3>🔒 TEE Security Guarantees</h3>
                    <p>Critical operations execute in isolated TEE environments with formally verified isolation properties</p>
                  </div>
                </div>
              </div>
            </section>

            {/* Implementation & Methodology */}
            <section className={styles.techSection}>
              <h2 className={styles.sectionTitle} style={{ color: 'white' }}>Research Methodology</h2>
              <div className={styles.techGrid}>
                <div className={styles.techItem}>
                  <strong>Formal Verification:</strong> Lean 4 theorem prover for key security lemmas and privacy properties on PASS formal model.
                </div>
                <div className={styles.techItem}>
                  <strong>Prototype Implementation:</strong> Rust enclave logic, TypeScript frontend, and WalletConnect v2 integration for EVM blockchains.
                </div>
                <div className={styles.techItem}>
                  <strong>TEE Infrastructure:</strong> AWS Nitro Enclaves and Phala dStackfor secure key management and execution.
                </div>
              </div>
            </section>

            {/* Funding Acknowledgment */}
            <section className={styles.fundingSection}>
              <div className={styles.fundingContent}>
                <h2 className={styles.sectionTitle}>Funding Acknowledgment</h2>
                <p className={styles.fundingText}>
                  This work was funded in part by the <strong>Ethereum Foundation Academic Grants Round 2025</strong>.
                </p>
              </div>
            </section>
          </>
        ) : (
          <div className={styles.welcomeContainer}>
            <div className={styles.connectedHeader}>
              <h1>Welcome back to PassWallet</h1>
              <p className={styles.connectedAccount}>Connected: <span>{address}</span></p>
            </div>
            <AccountList />
          </div>
        )}
      </main>

      {/* <footer className={styles.footer}>
        <p>Built with ❤️ for secure decentralized finance • Powered by AWS Nitro Enclaves</p>
      </footer> */}
    </div>
  );
};

export default Home;