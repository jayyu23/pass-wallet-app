import { ConnectButton } from '@rainbow-me/rainbowkit';
import type { NextPage } from 'next';
import Head from 'next/head';
import styles from '../styles/Home.module.css';
import { useAccount } from 'wagmi';
import Navbar from '../components/Navbar';
import AccountList from '../components/AccountList';

const Home: NextPage = () => {
  const { address, isConnected } = useAccount();

  const features = [
    {
      title: "TEE-Based Security",
      description: "Keys are managed within AWS Nitro Enclaves, ensuring secure execution isolated from the host system.",
      icon: "🔐"
    },
    {
      title: "Multi-Account Support",
      description: "Create and manage multiple Ethereum accounts with sophisticated access controls.",
      icon: "👥"
    },
    {
      title: "WalletConnect Integration",
      description: "Seamlessly connect and interact with dApps using WalletConnect v2 protocol.",
      icon: "🔗"
    },
    {
      title: "Asset Management",
      description: "View balances, transfer assets, and manage your crypto portfolio across accounts.",
      icon: "💰"
    },
    {
      title: "Message Signing",
      description: "Sign messages securely through the enclave with cryptographic verification.",
      icon: "✍️"
    },
    {
      title: "Transaction History",
      description: "Track all your transactions and signed messages with detailed audit logs.",
      icon: "📊"
    }
  ];

  return (
    <div className={styles.container}>
      <Head>
        <title>PassWallet - Secure Decentralized Wallet with TEE</title>
        <meta name="description" content="PassWallet is a decentralized wallet application that enables secure key management through Trusted Execution Environments (TEE)" />
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
                  PassWallet
                  <span className={styles.subtitle}>Secure Crypto Wallet with TEE Protection</span>
                </h1>
                <p className={styles.heroDescription}>
                  Experience next-generation wallet security with PassWallet. Built on AWS Nitro Enclaves, 
                  our decentralized wallet application provides uncompromising key management through 
                  Trusted Execution Environments (TEE).
                </p>
                <div className={styles.connectContainer}>
                  <ConnectButton />
                  <p className={styles.connectHint}>Connect your wallet to start exploring secure crypto management</p>
                </div>
              </div>
            </div>

            {/* Features Section */}
            <section className={styles.featuresSection}>
              <h2 className={styles.sectionTitle}>Why Choose PassWallet?</h2>
              <div className={styles.featuresGrid}>
                {features.map((feature, index) => (
                  <div key={index} className={styles.featureCard}>
                    <div className={styles.featureIcon}>{feature.icon}</div>
                    <h3 className={styles.featureTitle}>{feature.title}</h3>
                    <p className={styles.featureDescription}>{feature.description}</p>
                  </div>
                ))}
              </div>
            </section>

            {/* Security Section */}
            <section className={styles.securitySection}>
              <div className={styles.securityContent}>
                <h2 className={styles.sectionTitle}>Enterprise-Grade Security</h2>
                <div className={styles.securityGrid}>
                  <div className={styles.securityItem}>
                    <h3>🛡️ Key Encumbrance</h3>
                    <p>Advanced key management with predefined rules that cannot be arbitrarily overwritten</p>
                  </div>
                  <div className={styles.securityItem}>
                    <h3>🔒 TEE Isolation</h3>
                    <p>Critical operations execute in isolated environments, protected from host system access</p>
                  </div>
                  <div className={styles.securityItem}>
                    <h3>⚡ Zero Trust Architecture</h3>
                    <p>Built on principles of never trusting, always verifying every transaction and operation</p>
                  </div>
                </div>
              </div>
            </section>

            {/* Technology Stack */}
            <section className={styles.techSection}>
              <h2 className={styles.sectionTitle}>Built with Modern Technology</h2>
              <div className={styles.techGrid}>
                <div className={styles.techItem}>
                  <strong>Frontend:</strong> Next.js, RainbowKit, WalletConnect v2
                </div>
                <div className={styles.techItem}>
                  <strong>Security:</strong> AWS Nitro Enclaves, TEE Architecture
                </div>
                <div className={styles.techItem}>
                  <strong>Blockchain:</strong> Ethereum, Multi-chain Support
                </div>
                <div className={styles.techItem}>
                  <strong>Database:</strong> Prisma ORM, SQLite
                </div>
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

      <footer className={styles.footer}>
        <p>Built with ❤️ for secure decentralized finance • Powered by AWS Nitro Enclaves</p>
      </footer>
    </div>
  );
};

export default Home;