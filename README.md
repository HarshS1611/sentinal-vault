# Sentinel Vault 

A secure **Anchor-based Solana vault** that enforces **activity-based and cooldown-based withdrawal constraints**.
Users must periodically check in to remain eligible for withdrawals, and withdrawals are rate-limited to prevent misuse or abandoned vault draining.

---

## ✨ Features

* Built with **Anchor**
* **Unique constraint**:

  * Withdrawals allowed only if:

    * User has checked in within an inactivity window
    * Cooldown period since last withdrawal has passed
    * Withdrawal amount ≤ deposited amount
    * Caller is the vault owner
* Full **positive & negative test coverage**
* Deployed on **Solana Devnet**

---

## 📦 Instructions

* `initialize` – Create vault with cooldown & inactivity window
* `deposit` – Deposit SOL into the vault
* `check_in` – Mark user as active
* `withdraw` – Withdraw SOL if all constraints are satisfied

---

## 🧪 Tests

All tests are written using **Anchor + Mocha** and executed against a **local validator**.

```bash
anchor test
```

---

## 🚀 Devnet Deployment

* **Cluster:** Devnet

* **Program ID:**

  ```
  2QLVKGpugTttecUSjjt4kERsVVrmhyzqMR6N5Cdp6q1H
  ```

* **Solana Explorer:**
  [https://explorer.solana.com/address/2QLVKGpugTttecUSjjt4kERsVVrmhyzqMR6N5Cdp6q1H?cluster=devnet](https://explorer.solana.com/address/2QLVKGpugTttecUSjjt4kERsVVrmhyzqMR6N5Cdp6q1H?cluster=devnet)

---

## 📂 Screenshots

> Test cases passed
<img width="1938" height="1966" alt="image" src="https://github.com/user-attachments/assets/6284a26a-911b-4a2d-8073-4acd0202eeb0" />

> Deployed on devnet
<img width="1938" height="1828" alt="image" src="https://github.com/user-attachments/assets/f09fa9f0-59f1-4556-a39f-6108bfc0bd13" />

---

## 🧠 Constraint Logic (Brief)

Sentinel Vault prevents unattended or automated fund extraction by requiring:

* **Proof of activity** via `check_in`
* **Time-based cooldowns** between withdrawals

This design ensures funds remain safe even if the owner becomes inactive.

