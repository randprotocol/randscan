import type { Metadata } from 'next';
import Link from 'next/link';
import { LegalPage, type LegalSection } from '@/components/LegalPage';

export const metadata: Metadata = {
  title: 'Terms of Service — RandScan',
  description: 'The terms on which RandScan, the Rand Protocol explorer, its API and its accounts are offered.',
};

const EFFECTIVE = '1 October 2026';

const sections: LegalSection[] = [
  {
    id: 'agreement',
    title: 'The agreement',
    body: (
      <>
        <p>
          These terms govern your use of RandScan (randscan.org): the website, the JSON API at
          randscan.org/api, the WebSocket feed, and the accounts and API keys offered through it.
          RandScan is run by the Rand Protocol team (&ldquo;we&rdquo;). By using any of it you accept
          these terms. If you do not accept them, do not use the service.
        </p>
        <p>
          The <Link href="/privacy" className="link">Privacy Policy</Link> explains what the service
          collects and is part of these terms.
        </p>
      </>
    ),
  },
  {
    id: 'service',
    title: 'What RandScan is, and is not',
    body: (
      <>
        <p>
          RandScan is a block explorer. It reads the public ledger of the Rand Protocol network from a
          full node, indexes it, and presents it. It is a window onto the chain, not part of it.
        </p>
        <ul className="list-disc space-y-1 ps-5">
          <li>
            It is not a wallet. It holds no funds, no spending keys and no custody of anything, and
            it cannot create, sign or broadcast transactions on your behalf.
          </li>
          <li>
            It is not the bridge. The <Link href="/bridge" className="link">Bridge</Link> page
            describes bridge activity recorded on the Rand chain. The bridge itself is run by its
            guardians under their own terms; deposits, approvals and settlement happen on the source
            chains, and should be verified there.
          </li>
          <li>
            It is not advice. Nothing on RandScan is financial, investment, legal or tax advice, and
            nothing is an offer or solicitation to buy or sell any asset.
          </li>
        </ul>
      </>
    ),
  },
  {
    id: 'accuracy',
    title: 'Accuracy of the data',
    body: (
      <>
        <p>
          We try to show the chain faithfully, and the explorer&rsquo;s code is open for anyone to
          check. Still, what you see is a copy made by software, and it can be wrong or late:
        </p>
        <ul className="list-disc space-y-1 ps-5">
          <li>the indexer can lag behind the chain, or stop while the node it reads is restarted;</li>
          <li>a block the explorer showed can be replaced if the chain reorganises;</li>
          <li>
            when the network moves to a new chain, the explorer&rsquo;s history is reset to the new
            genesis, and the previous chain&rsquo;s data is no longer served;
          </li>
          <li>
            a shielded chain hides amounts and parties by design, so the explorer necessarily shows
            only what the ledger makes public, and labels such as token names come from the
            explorer&rsquo;s own lists, not from the chain;
          </li>
          <li>node and prover locations, versions and status are observations, not guarantees.</li>
        </ul>
        <p>
          Decisions that depend on the state of the chain should be checked against a node you run or
          trust. We make no promise that any figure on RandScan is correct or current.
        </p>
      </>
    ),
  },
  {
    id: 'viewing',
    title: 'Viewing keys and the note opener',
    body: (
      <>
        <p>
          The <Link href="/viewing" className="link">History</Link> page opens shielded notes with a
          key you paste. The opening runs in your browser and the key is not sent to us, as the
          Privacy Policy describes. You remain responsible for the keys you handle: paste them only
          on a device and browser you trust, prefer a viewing key to a spend key wherever one will do,
          and understand that we cannot recover a key or undo what someone who obtained it can see.
          Results of the opener are a best-effort reading of the chain and carry the same caveats as
          every other figure here.
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
          You do not need an account to use RandScan. An account issues API keys with a higher
          request quota. If you create one:
        </p>
        <ul className="list-disc space-y-1 ps-5">
          <li>you must give an email address you control, and keep your password secret;</li>
          <li>
            an API key is a credential. Keep it out of client-side code and public repositories, and
            revoke it from your <Link href="/dashboard" className="link">dashboard</Link> if it leaks.
            Everything done with your key counts as done by you;
          </li>
          <li>you may hold up to 10 active keys, and one account per person or system is the norm;</li>
          <li>
            you may not share, sell or pool keys to exceed the quota, or create accounts by automated
            means.
          </li>
        </ul>
        <p>
          We may suspend or delete an account, or revoke its keys, if it breaches these terms, is
          used abusively, or has been inactive for a long time. You may stop using the service and ask
          us to delete your account at any time.
        </p>
      </>
    ),
  },
  {
    id: 'quotas',
    title: 'Quotas and acceptable use',
    body: (
      <>
        <p>
          Public read endpoints are free. Anonymous traffic is limited per IP address and keyed
          traffic per key; the current figures are published in the{' '}
          <a
            href="https://github.com/randprotocol/randscan.org/blob/main/docs/api.md"
            target="_blank"
            rel="noopener noreferrer"
            className="link"
          >
            API documentation
          </a>{' '}
          and may change. The service answers over the limit with HTTP 429 and a Retry-After header,
          and clients are expected to honour it.
        </p>
        <p>You agree not to:</p>
        <ul className="list-disc space-y-1 ps-5">
          <li>
            evade rate limits, for instance by rotating addresses or keys, or by many accounts;
          </li>
          <li>
            probe, scan, overload or otherwise interfere with the service, its servers or the node
            behind it;
          </li>
          <li>
            use the service to break the law, to infringe anyone&rsquo;s rights, or to harass or
            de-anonymise people;
          </li>
          <li>
            present RandScan&rsquo;s data as your own without attribution, or imply that we endorse a
            product that uses it.
          </li>
        </ul>
        <p>
          Reading the public data in bulk is fine within the quotas. If you need more, run your own
          node and indexer; the software is free.
        </p>
      </>
    ),
  },
  {
    id: 'ip',
    title: 'Software, data and marks',
    body: (
      <>
        <p>
          The explorer&rsquo;s source code is released under the MIT licence, and you may use it on
          those terms. Chain data shown on RandScan is public and belongs to no one; take it, with a
          mention of where it came from. The RandScan name and mark and the Rand Protocol name and
          mark identify this service and the network, and may not be used to suggest that something
          is ours when it is not.
        </p>
      </>
    ),
  },
  {
    id: 'third-parties',
    title: 'Third parties',
    body: (
      <>
        <p>
          RandScan links to other sites and chains, and the map pages load base-map tiles from
          Esri&rsquo;s ArcGIS Online. Those services have their own terms. We do not control them and
          are not responsible for what they do or show.
        </p>
      </>
    ),
  },
  {
    id: 'warranty',
    title: 'No warranty',
    body: (
      <>
        <p>
          RandScan is provided &ldquo;as is&rdquo; and &ldquo;as available&rdquo;, without warranty
          of any kind, express or implied, including of accuracy, availability, fitness for a
          particular purpose or non-infringement. It is run by a small team on a best-effort basis;
          it may be unavailable, may change, and may be withdrawn, with or without notice.
        </p>
      </>
    ),
  },
  {
    id: 'liability',
    title: 'Limitation of liability',
    body: (
      <>
        <p>
          To the fullest extent the law allows, we are not liable for any loss or damage arising from
          your use of, or inability to use, RandScan or anything shown on it. That includes loss of
          funds, lost profits, loss of data, and any indirect or consequential loss, however caused,
          and even if we were told it was possible. In particular, we are not liable for decisions
          taken on the basis of explorer data, for transactions on any chain, or for the acts of
          third parties. Where liability cannot be excluded, it is limited to the amount you paid us
          for the service, which is nothing.
        </p>
        <p>
          Some jurisdictions do not allow certain exclusions. Nothing here takes away rights you have
          under law that cannot be waived.
        </p>
      </>
    ),
  },
  {
    id: 'changes',
    title: 'Changes to these terms',
    body: (
      <>
        <p>
          We may change these terms. The current text is always at randscan.org/terms with its
          effective date. If a change is material we will say so on the explorer before it takes
          effect. Using the service after a change means you accept the new terms. This version took
          effect on {EFFECTIVE}.
        </p>
      </>
    ),
  },
  {
    id: 'general',
    title: 'General',
    body: (
      <>
        <p>
          If any part of these terms is found unenforceable, the rest still applies. Our not acting on
          a breach is not a waiver. These terms and the Privacy Policy are the whole agreement between
          you and us about RandScan. Questions go to the Rand Protocol team through{' '}
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
      </>
    ),
  },
];

export default function TermsPage() {
  return (
    <LegalPage
      title="Terms of Service"
      effective={EFFECTIVE}
      intro="RandScan is a free, open-source window onto a public ledger. These terms say what the service is, what it is not, how its API and accounts may be used, and the limits of what we promise."
      sections={sections}
      related={{ href: '/privacy', label: 'Privacy Policy' }}
    />
  );
}
