import { ConnectButton } from '@rainbow-me/rainbowkit';
import styles from '../styles/Navbar.module.css';

const Navbar = () => {
  return (
    <nav className={styles.navbar}>
      <div className={styles.leftSection}>
        <div className={styles.logo}>
          <a href="/" style={{ textDecoration: 'none', color: 'inherit' }}>
            📘 PassWallet
          </a>
        </div>
        <div className={styles.navLinks}>
          {/* <a href="/" className={styles.navLink}>Send ETH</a>
          <a href="/mint" className={styles.navLink}>Mint NFT</a> */}
        </div>
      </div>
      
      <div className={styles.walletButton}>
        <ConnectButton />
      </div>
    </nav>
  );
};

export default Navbar;