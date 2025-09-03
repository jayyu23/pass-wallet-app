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
          <a href="#" className={styles.navLink} target="_blank" rel="noopener noreferrer">Paper</a>
          <a href="#" className={styles.navLink} target="_blank" rel="noopener noreferrer">App Repo</a>
          <a href="#" className={styles.navLink} target="_blank" rel="noopener noreferrer">Formal Verification</a>
        </div>
      </div>
      
      <div className={styles.walletButton}>
        <ConnectButton />
      </div>
    </nav>
  );
};

export default Navbar;