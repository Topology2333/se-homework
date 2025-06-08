<template>
  <div>
    <h2>充电桩状态</h2>
    <el-table :data="pileList" style="width: 100%" v-loading="loading">
      <el-table-column prop="id" label="编号" width="100"/>
      <el-table-column prop="type" label="类型" width="120">
        <template #default="{ row }">
          <el-tag :type="row.type === 'fast' ? 'warning' : ''">
            {{ row.type === 'fast' ? '快充' : '慢充' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="state" label="运行状态" width="120">
        <template #default="{ row }">
          <el-tag :type="row.state ? 'success' : 'danger'">
            {{ row.state ? '运行中' : '已关闭' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="totalChargeCount" label="累计充电次数" width="150"/>
      <el-table-column prop="totalChargeTime" label="累计时长(分钟)" width="150"/>
      <el-table-column prop="totalPower" label="累计电量(度)" width="150"/>
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
    fetchPiles() {
      this.loading = true;
      axios.get('/api/admin/status')
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
    }
  }
}
</script>