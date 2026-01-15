# Vaultix

A modern escrow-style application that protects buyers and sellers by holding funds securely until all transaction conditions are met. This platform ensures trust, transparency, and security for online transactions.

## Features

* Secure fund holding system
* Buyer & seller protection
* Transaction milestone tracking
* Automated release of funds
* Dispute resolution system
* User authentication & authorization
* Real-time transaction status updates
* Admin dashboard for monitoring



## Tech Stack

**Frontend:**

* Next.js
* TypeScript
* Tailwind CSS

**Backend:**

* Node.js / NestJS
* PostgreSQL
* Prisma ORM

**Blockchain Layer:**

* **Stellar Blockchain**
* Stellar SDK (JS)


**Authentication:**

* JWT / OAuth

**Payments:**

* Stellar Lumens (XLM)
* Custom Stellar assets (optional)



##  Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/project-name.git

# Navigate into project
cd project-name

# Install dependencies
npm install

# Run development server
npm run dev
```

---

## Environment Variables

Create a `.env` file and add:

```env
DATABASE_URL=
JWT_SECRET=
PAYMENT_API_KEY=
NEXT_PUBLIC_APP_URL=
```

---

## How It Works

1. Buyer initiates a transaction
2. Funds are locked on the Stellar blockchain
3. Smart contract escrow logic is triggered
4. Seller delivers service/product
5. Buyer confirms satisfaction
6. Funds are released automatically via Stellar

---

## 👥 User Roles

* Buyer
* Seller
* Admin

---

## 📁 Project Structure

```
/project
 ├── app
 ├── components
 ├── lib
 ├── pages
 ├── public
 ├── styles
 └── prisma
```



## Admin Capabilities

* View all transactions
* Freeze suspicious accounts
* Resolve disputes
* Analytics dashboard



##  Security Measures

* On-chain transaction verification
* Smart contract escrow logic
* Multi-signature wallets
* Encrypted API communication
* Two-factor authentication
* Audit logs



##  Testing

```bash
npm run test
```

---

## Deployment

* Vercel (Frontend)
* AWS / Render (Backend)

---

## Contribution

1. Fork the project
2. Create a new branch
3. Commit changes
4. Push to branch
5. Open a pull request



##  License

MIT License



##  Vision

To build a secure, decentralized transaction infrastructure powered by Stellar blockchain that promotes trust and fairness in digital payments across Africa and beyond.
