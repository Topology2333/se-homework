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
      <el-table-column label="操作" width="250">
        <template #default="{ row }">
          <el-button size="mini" type="info" @click="showWaitingRequests(row)">查看排队</el-button>
          <el-button size="mini" type="success" @click="startPile(row.id)">启动</el-button>
          <el-button size="mini" type="danger" @click="shutdownPile(row.id)">关闭</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 排队信息对话框 -->
    <el-dialog 
      :title="`充电桩 ${currentPileNumber} 排队车辆信息`" 
      v-model="waitingDialogVisible" 
      width="60%">
      <el-table :data="waitingRequests" style="width: 100%">
        <el-table-column prop="user_id" label="用户ID" width="350" />
        <el-table-column prop="amount" label="请求电量(度)" width="150" />
        <el-table-column prop="status" label="状态" width="150" />
        <el-table-column label="排队时长" width="150">
          <template #default="{ row }">
            {{ formatQueueDuration(row.created_at) }}
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>
  </div>
</template>

<script>
import axios from 'axios';
export default {
  name: 'AdminPage',
  data() {
    return {
      pileList: [],
      loading: false,
      waitingDialogVisible: false,
      waitingRequests: [],
      currentPileId: null,
      currentPileNumber: ''
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
    },

    // 显示排队信息
    showWaitingRequests(pile) {
      this.currentPileId = pile.id;
      this.currentPileNumber = pile.number;
      this.waitingDialogVisible = true;
      this.fetchWaitingRequests();
    },

    // 获取排队信息
  fetchWaitingRequests() {
    axios.get(`http://localhost:8080/piles/${this.currentPileId}/waiting-requests`)
      .then(res => {
        this.waitingRequests = res.data;
      })
      .catch(err => {
        console.error("获取排队信息失败:", err.response); // 打印完整错误响应
        this.$message.error('获取排队信息失败: ' + (err.response?.data || err.message));
      });
  },

    // 格式化排队时长
    formatQueueDuration(createdAt) {
        const created = new Date(createdAt);
        const now = new Date();
        const nowAdjusted = new Date(now.getTime() + 8 * 3600 * 1000);
        // 使用 UTC 时间计算
        const diffSec = Math.floor((nowAdjusted.getTime() - created.getTime()) / 1000);

        if (diffSec < 0) return "刚刚";
        if (diffSec < 60) return `${diffSec} 秒`;
        if (diffSec < 3600) return `${Math.floor(diffSec / 60)} 分钟`;
        
        const h = Math.floor(diffSec / 3600);
        const m = Math.floor((diffSec % 3600) / 60);
        return `${h} 小时 ${m} 分`;
    }
  }
}
</script>

<style scoped>
.el-button + .el-button {
  margin-left: 8px;
}
</style>