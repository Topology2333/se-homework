<template>
  <div class="home-container">
    <el-card class="box-card" shadow="hover" style="height: 50vh; width: 25vw; text-align: center; padding: 20px;">
      <h2>欢迎新用户</h2>

      <el-form :model="form" label-position="top" style="width: 100%; max-width: 400px; margin: 50px auto;">
        <el-form-item>
          <el-input v-model="username" placeholder="用户名" />
        </el-form-item>

        <el-form-item>
          <el-input v-model="password" type="password" placeholder="密码" />
        </el-form-item>

        <el-form-item style="width: 100%;">
          <el-button type="success" @click="register" style="width: 100%;">注册</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script>
import axios from 'axios'
export default {
  name: 'UserRegister',
  data: () => ({
    username: '',
    password: ''
  }),
  methods: {
    async register() {
      try {
        await axios.post('http://localhost:8080/register', {
          username: this.username,
          password: this.password,
          is_admin: false
        })
        alert('注册成功')
        this.$router.push('/')
      } catch {
        alert('注册失败')
      }
    }
  }
}
</script>

<style>
.home-container {
  height: 90vh;
  width: 50vw;
  margin: auto;
  display: flex;
  justify-content: center;
  align-items: center;
  background: transparent;
  background-color: rgba(255, 255, 255, 0.8);
}
</style>