/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'standalone',
  async rewrites() {
    // With no NEXT_PUBLIC_API_URL the frontend talks to the API on its own
    // origin (Caddy proxies /api and /ws), so no rewrite is needed.
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
