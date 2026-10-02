/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  // The middleware's request URL (and so every redirect it issues) is built from the Host header
  // instead of the listen address: behind Caddy the listen address is 127.0.0.1:3001, and a
  // redirect to it is useless to a browser. Caddy keeps the original Host.
  experimental: { trustHostHeader: true },
  async redirects() {
    // /tx/<hash> is the short form wallets link to ("Open in randscan" in Rand Wallet
    // v0.6.8 and earlier); the page lives at /transactions/<hash>. A 0x prefix is
    // dropped here; the API lowercases the hash and accepts either form anyway.
    return [
      { source: '/tx/0x:hash', destination: '/transactions/:hash', permanent: true },
      { source: '/tx/:hash', destination: '/transactions/:hash', permanent: true },
    ];
  },
  async rewrites() {
    // The browser always calls the same-origin /api. This rewrite forwards
    // those calls to NEXT_PUBLIC_API_URL when it's set, which is how the
    // API is reached in development and in Docker (rewrites are evaluated
    // at build time, so the destination must be baked in there). In
    // production Caddy proxies /api directly to the API, so
    // NEXT_PUBLIC_API_URL is left unset and no rewrite is needed.
    const apiUrl = (process.env.NEXT_PUBLIC_API_URL || '').trim();
    if (!apiUrl) return [];

    return [
      {
        source: '/api/:path*',
        destination: `${apiUrl.replace(/\/+$/, '')}/api/:path*`,
      },
    ];
  },
};

module.exports = nextConfig;
