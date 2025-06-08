<template>
  <div
    style="width: 70vw;margin: 0 auto; padding: 20px;background: transparent;background-color:  rgba(255, 255, 255, 0.8);">
    <el-header style="font-size: 24px; font-weight: bold; text-align: center; margin-bottom: 20px;">
      充电桩状态
    </el-header>
    <el-table :data="pileList" style="width: 100%" v-loading="loading">
      <el-table-column prop="number" label="编号" width="100" />
      <el-table-column prop="mode" label="类型" width="120">
        <template #default="{ row }">
          <el-tag :type="row.mode === 'Fast' ? 'warning' : ''">
            {{ row.mode === 'Fast' ? '快充' : '慢充' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="status" label="运行状态" width="120">
        <template #default="{ row }">
          <el-tag :type="row.status === 'Charging' || row.status === 'Available' ? 'success' : 'danger'">
            {{ row.status }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="total_charge_count" label="累计充电次数" width="150" />
      <el-table-column prop="total_charge_time" label="累计时长(小时)" width="150" />
      <el-table-column prop="total_charge_amount" label="累计电量(度)" width="150" />
      <el-table-column prop="total_charging_fee" label="累计费用(元)" width="120" />
      <el-table-column label="操作" width="200">
        <template #default="{ row }">
          <el-button size="mini" type="success" @click="startPile(row.id)">启动</el-button>
          <el-button size="mini" type="danger" @click="shutdownPile(row.id)">关闭</el-button>
        </template>
      </el-table-column>
    </el-table>
  </div>
</template>

<script>
import axios from 'axios';

export default {
  name: 'AdminPage',
  data() {
    return {
      pileList: [],
      loading: false
    };
  },
  mounted() {
    this.fetchPiles();
  },
  methods: {
    // 获取充电桩列表
    fetchPiles() {
      this.loading = true;
      axios.get('http://localhost:8080/piles')
        .then(res => {
          this.pileList = res.data;
        })
        .catch(err => {
          console.error("获取失败:", err);
          this.$message.error('获取充电桩状态失败');
        })
        .finally(() => {
          this.loading = false;
        });
    },

    // 启动充电桩
    startPile(id) {
      axios.post(`http://localhost:8080/piles/${id}/start`)
        .then(() => {
          this.$message.success('启动成功');
          this.fetchPiles(); // 刷新状态
        })
        .catch(err => {
          console.error(err);
          this.$message.error('启动失败');
        });
    },

    // 关闭充电桩
    shutdownPile(id) {
      axios.post(`http://localhost:8080/piles/${id}/shutdown`)
        .then(() => {
          this.$message.success('关闭成功');
          this.fetchPiles(); // 刷新状态
        })
        .catch(err => {
          console.error(err);
          this.$message.error('关闭失败');
        });
    }
  }
}
</script>
