export function Footer() {
  return (
    <footer className="border-t border-border">
      <div className="mx-auto flex max-w-6xl flex-col items-start justify-between gap-3 px-6 py-8 text-[0.8125rem] text-mute sm:flex-row sm:items-center lg:px-8">
        <p>RandScan — explorer for Rand Protocol</p>
        <div className="flex items-center gap-5">
          <a
            href="https://randprotocol.org"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-strong"
          >
            randprotocol.org
          </a>
          <a
            href="https://github.com/randprotocol"
            target="_blank"
            rel="noopener noreferrer"
            className="transition-colors hover:text-strong"
          >
            GitHub
          </a>
        </div>
      </div>
    </footer>
  );
}
