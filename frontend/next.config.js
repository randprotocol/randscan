/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
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
