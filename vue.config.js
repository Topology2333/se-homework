// vue.config.js
const { defineConfig } = require('@vue/cli-service')

module.exports = defineConfig({
  devServer: {
    port: 9876,
    proxy: {
      '/api': {  // 关键：必须与请求路径前缀一致
        target: 'http://localhost:8080',// 后端API
        changeOrigin: true,
        // 会报错
        // pathRewrite: {
        //   '^/api': '' // 如果后端API路径没有前缀，可以去掉/api
        // }
      }
    }
  }
})