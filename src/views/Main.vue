<template>
  <el-container style="width: 60vw;margin: 0 auto; padding: 20px;background: transparent;background-color:  rgba(255, 255, 255, 0.8);">
    <el-header style="font-size: 24px; font-weight: bold; text-align: center; margin-bottom: 20px;">
      用户主页 - 欢迎，{{ username }}
    </el-header>

    <el-main style="">
      <el-row :gutter="20">

        <el-col :span="12">
          <el-card shadow="hover">
            <h3>等候区</h3>
            <p><strong>您的排队号码：</strong> {{ queueNumber }}</p>
            <p><strong>充电模式：</strong> {{ modeText }}</p>
            <p><strong>排队前车辆数：</strong> {{ waitingCount }}</p>
            <el-button type="primary" @click="goToRequest" style="margin-top: 10px;">提交充电请求</el-button>
          </el-card>
        </el-col>

        <el-col :span="12">
          <el-card shadow="hover">
            <h3>充电区</h3>
            <p>快充电桩数量：2，功率：30度/小时</p>
            <p>慢充电桩数量：3，功率：10度/小时</p>
            <el-button type="success" @click="goToDetails" style="margin-top: 10px;">查看充电详单</el-button>
          </el-card>
        </el-col>
      </el-row>
    </el-main>
  </el-container>
</template>

<script>
export default {
  name: "UserHome",
  data() {
    return {
      queueNumber: 'F3',
      mode: 'F', // F=快充，T=慢充
      waitingCount: 2,
      username: '用户'
    };
  },
  created() {
    // 从localStorage获取用户名
    const user = JSON.parse(localStorage.getItem('user'));
    if (user) {
      this.username = user.username;
    }
  },
  computed: {
    modeText() {
      return this.mode === 'F' ? '快充' : '慢充';
    },
  },
  methods: {
    goToRequest() {
      this.$router.push('/request');
    },
    goToDetails() {
      this.$router.push('/details');
    }
  }
};
</script>

<style scoped>
h3 {
  margin-bottom: 10px;
}
</style>