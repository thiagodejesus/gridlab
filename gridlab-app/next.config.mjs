/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config, ctx) => {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      syncWebAssembly: true,
    };
    return config;
  },
};

export default nextConfig;
