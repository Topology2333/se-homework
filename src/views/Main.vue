<template>
  <el-container
    style="width: 60vw; margin: 0 auto; padding: 20px; background-color: rgba(255, 255, 255, 0.9); border-radius: 12px;">
    <el-header style="font-size: 24px; font-weight: bold; text-align: center; margin-bottom: 20px;">
      智能充电桩调度系统 - 欢迎，{{ username }}
    </el-header>

    <el-main style="text-align: center;">
      <!-- 电价图表 -->
      <el-card shadow="hover" style="margin-bottom: 20px;">
        <h3 style="margin-bottom: 10px;">各时段电价（元/度）</h3>
        <v-chart :option="chartOptions" style="height: 300px;"></v-chart>
      </el-card>

      <el-main style="">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-card shadow="hover">
              <h3>充电模式</h3>
              <p><strong>快充电桩数量：</strong>2，功率：30度/小时</p>
              <p><strong>慢充电桩数量：</strong>3，功率：10度/小时</p>
            </el-card>
          </el-col>

          <el-col :span="12">
            <el-card shadow="hover">
              <h3>电价分布</h3>
              <ul style="text-align: left;"> 
                <li>峰时：10:00~15:00，18:00~21:00</li>
                <li>平时：7:00~10:00，15:00~18:00，21:00~23:00</li>
                <li>谷时：23:00~次日7:00</li>
              </ul>
            </el-card>
          </el-col>
        </el-row>
      </el-main>

      <!-- 操作按钮 -->
      <div style="margin-top: 20px;">
        <el-button type="primary" @click="goToRequest" style="margin-right: 10px;">提交充电请求</el-button>
        <el-button type="success" @click="goToDetails">查看充电详单</el-button>
      </div>
    </el-main>
  </el-container>
</template>

<script>
export default {
  name: "UserHome",
  data() {
    return {
      username: '用户',
      chartOptions: {
        tooltip: {
          trigger: 'axis'
        },
        xAxis: {
          type: 'category',
          data: ['峰时', '平时', '谷时']
        },
        yAxis: {
          type: 'value',
          name: '元/度'
        },
        series: [
          {
            data: [1.0, 0.7, 0.4],
            type: 'bar',
            barWidth: '40%',
            itemStyle: {
              borderRadius: 5
            }
          }
        ]
      }
    };
  },
  created() {
    const user = JSON.parse(localStorage.getItem('user'));
    if (user) {
      this.username = user.username;
    }
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
ul {
  padding-left: 1em;
  margin: 0;
}

ul li {
  margin-bottom: 4px;
}
</style>
