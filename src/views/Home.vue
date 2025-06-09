<template>
  <div class="home-container">
    <el-card class="box-card" shadow="hover" style="height: 50vh; width: 25vw; text-align: center; padding: 20px;">
      <h2>欢迎使用智能充电桩</h2>

      <el-form :model="form" label-position="top" style="width: 100%; max-width: 400px; margin: 50px auto;">
        <el-form-item>
          <el-input v-model="form.username" placeholder="用户名" />
        </el-form-item>

        <el-form-item>
          <el-input v-model="form.password" type="password" placeholder="密码" />
        </el-form-item>

        <el-form-item style="width: 100%;margin-top: 60px;">
          <el-button type="primary" @click="login" style="width: 100%; margin-bottom: 10px;">登录</el-button>
        </el-form-item>

        <el-form-item style="width: 100%;">
          <el-button type="success" @click="goRegister" style="width: 100%;">注册</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script>
import axios from 'axios'
export default {
  name: 'HomePage',
  data: () => ({
    form: {
      username: '',
      password: ''
    }
  }),
	// add console message
  methods: {
    async login() {
      try {
				console.log('尝试登录，用户名:', this.form.username)
        const response = await axios.post('http://localhost:8080/login', {
          username: this.form.username,
          password: this.form.password
        })
				console.log('登录成功，响应数据:', response.data)
        
				const user = response.data;
        console.log('登录成功，用户信息：', user);
        localStorage.setItem('user', JSON.stringify(user));
        this.$router.push('/main')
                
      } catch (error) {
        alert('登录失败')
      }
    },
    goRegister() {
      this.$router.push('/register')
    }
  }
}
</script>

<style scoped>
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

.form-inline {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  align-items: center;
  gap: 10px;
  margin-top: 20px;
}

.form-item-inline {
  flex: 1 1 150px;
  /* 最小宽度150px，自适应扩展 */
  min-width: 150px;
}

.button-group {
  display: flex;
  gap: 10px;
  flex: 1 1 300px;
  justify-content: center;
}
</style>
