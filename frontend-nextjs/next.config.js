/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  // API 代理配置
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:3000/api/:path*', // 代理到 Rust 后端
      },
      {
        source: '/auth/:path*',
        destination: 'http://localhost:3000/auth/:path*',
      },
      {
        source: '/query',
        destination: 'http://localhost:3000/query',
      },
      {
        source: '/transaction',
        destination: 'http://localhost:3000/transaction',
      },
    ]
  },
}

module.exports = nextConfig

