import { ConnectButton } from '@rainbow-me/rainbowkit';
import styles from '../styles/Navbar.module.css';

const Navbar = () => {
  return (
    <nav className={styles.navbar}>
      <div className={styles.leftSection}>
        <div className={styles.logo}>
          <a href="/" style={{ textDecoration: 'none', color: 'inherit' }}>
            📘 PASS
          </a>
        </div>
        <div className={styles.navLinks}>
          <a href="cs191w.stanford.edu/projects/yu,%20jay_systems%20191w.pdf" className={styles.navLink} target="_blank" rel="noopener noreferrer">Paper</a>
          <a href="https://github.com/jayyu23/pass-wallet-app" className={styles.navLink} target="_blank" rel="noopener noreferrer">App Repo</a>
          <a href="https://github.com/jayyu23/pass-lean4-proofs" className={styles.navLink} target="_blank" rel="noopener noreferrer">Formal Verification</a>
        </div>
      </div>
      
      <div className={styles.walletButton}>
        <ConnectButton />
      </div>
    </nav>
  );
};

export default Navbar;