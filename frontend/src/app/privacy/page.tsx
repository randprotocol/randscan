import type { Metadata } from 'next';
import Link from 'next/link';
import { LegalPage, type LegalSection } from '@/components/LegalPage';

export const metadata: Metadata = {
  title: 'Privacy Policy — RandScan',
  description: 'What RandScan collects, what it never sees, and how long it keeps what it has.',
};

const EFFECTIVE = '1 October 2026';

const sections: LegalSection[] = [
  {
    id: 'scope',
    title: 'Who we are and what this covers',
    body: (
      <>
        <p>
          RandScan (randscan.org) is the block explorer for Rand Protocol, run by the Rand Protocol
          team (&ldquo;we&rdquo;). It indexes the public ledger of the Rand Protocol network and
          shows it as web pages, a JSON API and a WebSocket feed. This policy covers the website,
          the API at randscan.org/api and the accounts you can create to get an API key.
        </p>
        <p>
          The explorer&rsquo;s source code is published under the MIT licence, so everything
          described here can be checked against what the software actually does.
        </p>
      </>
    ),
  },
  {
    id: 'no-tracking',
    title: 'What we do not do',
    body: (
      <>
        <p>
          RandScan runs no analytics, no advertising and no tracking scripts. Fonts are served from
          our own server. No third-party JavaScript is loaded on any page. There is no cookie banner
          because there is nothing to consent to: the only cookie is the sign-in session described
          below, and it is set only after you sign in.
        </p>
        <p>We do not sell, rent or share personal data with anyone for their own purposes.</p>
      </>
    ),
  },
  {
    id: 'chain-data',
    title: 'Public blockchain data',
    body: (
      <>
        <p>
          Almost everything RandScan shows is a copy of the Rand Protocol ledger: blocks,
          transactions, notes, validators, programs, tokens and bridge activity. That ledger is
          public by design and replicated by every node on the network. We did not collect it from
          you, we cannot alter it, and we cannot remove entries from it.
        </p>
        <p>
          Rand Protocol is a shielded chain. Transaction amounts, senders and recipients are hidden
          on the ledger itself, and the explorer shows only what the chain makes public: the
          commitments, nullifiers and proofs of each transaction, plus the public opening of notes
          the chain computes itself, such as fees and bridge mints.
        </p>
      </>
    ),
  },
  {
    id: 'viewing-keys',
    title: 'Viewing keys stay in your browser',
    body: (
      <>
        <p>
          The <Link href="/viewing" className="link">History</Link> page and the note opener let you
          paste a viewing key, a transaction key or a wallet key file to read your own shielded
          activity. The decryption runs inside the page, in a WebAssembly module compiled from the
          explorer&rsquo;s own code. A pasted key is held in the page&rsquo;s memory only. It is never
          included in any request to our servers, never written to a cookie or to local storage, and
          is gone when you close or reload the page.
        </p>
        <p>
          The page does fetch the encrypted envelopes it needs from our API, in the same way any
          visitor can. We therefore see which transactions a browser asked for, but not which of
          them opened or what they contained.
        </p>
      </>
    ),
  },
  {
    id: 'visitors',
    title: 'What every visit involves',
    body: (
      <>
        <p>
          Like any web server, ours receives your IP address, the page or endpoint you asked for and
          the headers your browser sends. We use the IP address for two things:
        </p>
        <ul className="list-disc space-y-1 pl-5">
          <li>
            Rate limiting. Anonymous traffic is limited per IP address. The counter is held in the
            server&rsquo;s memory for a window of one minute and is not written to a database.
          </li>
          <li>
            Operating the service. Server and process logs may record requests while we diagnose
            problems. They stay on the server and are rotated away after a short time.
          </li>
        </ul>
        <p>
          Your theme choice (dark or light) is kept in your browser&rsquo;s local storage under the
          key <code className="font-mono text-xs">randscan-theme</code>. It never leaves your
          device.
        </p>
      </>
    ),
  },
  {
    id: 'map',
    title: 'The map tiles on Nodes and Provers',
    body: (
      <>
        <p>
          The <Link href="/nodes" className="link">Nodes</Link> and{' '}
          <Link href="/provers" className="link">Provers</Link> pages draw a world map. The base map
          tiles are loaded by your browser directly from Esri&rsquo;s ArcGIS Online tile service, so
          Esri receives your IP address and the tile coordinates in the same way it would for any
          map. No other page loads anything from a third party.
        </p>
        <p>
          The markers on that map locate the network&rsquo;s peers and the delegated provers&rsquo;
          hosts. Those are server addresses the network itself publishes, not visitors. Our indexer
          looks them up through the ipwho.is geolocation service and caches the city, country and
          provider it returns. Visitors&rsquo; addresses are never geolocated.
        </p>
      </>
    ),
  },
  {
    id: 'accounts',
    title: 'Accounts and API keys',
    body: (
      <>
        <p>
          You can use everything on RandScan without an account. An account exists only to issue API
          keys with a higher request quota. If you create one, we store:
        </p>
        <ul className="list-disc space-y-1 pl-5">
          <li>your email address and the time the account was created;</li>
          <li>
            an Argon2id hash of your password. We cannot read the password itself, and we never
            send it anywhere;
          </li>
          <li>
            for each API key, its name, the first characters of the key for you to recognise it, a
            SHA-256 hash of the secret, when it was created and when it was last used. The secret is
            shown once and is not stored;
          </li>
          <li>
            a sign-in session: a random token in the{' '}
            <code className="font-mono text-xs">randscan_session</code> cookie, HttpOnly and
            SameSite, with a hash of it in our database. A session lasts 30 days from its last use
            and is deleted when you sign out.
          </li>
        </ul>
        <p>
          Requests made with an API key are counted against that key for rate limiting, in the same
          one-minute memory window as anonymous traffic. We do not record which endpoints a key
          called or what it asked for.
        </p>
      </>
    ),
  },
  {
    id: 'email',
    title: 'Email',
    body: (
      <>
        <p>
          We send exactly one kind of email: a password-reset link, and only when you ask for one.
          It is delivered through Resend (resend.com), which processes your email address and the
          message to deliver it. The reset link expires after one hour. We do not send newsletters or
          notifications, and we do not use your address for anything else.
        </p>
      </>
    ),
  },
  {
    id: 'hosting',
    title: 'Where data is held',
    body: (
      <>
        <p>
          RandScan runs on servers rented from DigitalOcean, in Singapore. The database holding
          accounts, hashed credentials and sessions lives on that server. Backups, where taken, hold
          the same data for the same purposes. By using the service you accept that your data is
          processed there, wherever you are.
        </p>
      </>
    ),
  },
  {
    id: 'retention',
    title: 'How long we keep things',
    body: (
      <>
        <ul className="list-disc space-y-1 pl-5">
          <li>Rate-limit counters: one minute, in memory.</li>
          <li>Server logs: a short period, then rotated away.</li>
          <li>Sessions: 30 days from last use, or until you sign out.</li>
          <li>Password-reset tokens: one hour.</li>
          <li>Accounts and API keys: until you ask us to delete them.</li>
          <li>Public chain data: for as long as the explorer runs. It is the ledger.</li>
        </ul>
      </>
    ),
  },
  {
    id: 'rights',
    title: 'Your choices and rights',
    body: (
      <>
        <p>
          You can revoke any API key and sign out of any session from your{' '}
          <Link href="/dashboard" className="link">dashboard</Link>. To see the data held about your
          account, to correct it, or to have the account and its keys deleted,{' '}
          <a href="#contact" className="link">contact us</a> from the email address on the account. We will act within 30 days. There is not yet a
          self-service delete button; we will add one.
        </p>
        <p>
          Depending on where you live you may have further rights under laws such as the GDPR or the
          CCPA, including the right to complain to a supervisory authority. Nothing here limits them.
        </p>
      </>
    ),
  },
  {
    id: 'contact',
    title: 'Contact and changes',
    body: (
      <>
        <p>
          Questions about this policy or requests about your account go to the Rand Protocol team
          through the explorer&rsquo;s repository at{' '}
          <a
            href="https://github.com/randprotocol/randscan.org/issues"
            target="_blank"
            rel="noopener noreferrer"
            className="link"
          >
            github.com/randprotocol/randscan.org
          </a>
          .
        </p>
        <p>
          When this policy changes, the new text appears here with a new effective date. Changes
          that reduce your privacy will be announced on the explorer before they take effect. This
          version took effect on {EFFECTIVE}.
        </p>
      </>
    ),
  },
];

export default function PrivacyPage() {
  return (
    <LegalPage
      title="Privacy Policy"
      effective={EFFECTIVE}
      intro="RandScan shows a public ledger and keeps almost nothing about the people who look at it. This page says exactly what the explorer sees, what it stores, what it never sees, and how long anything is kept."
      sections={sections}
      related={{ href: '/terms', label: 'Terms of Service' }}
    />
  );
}
