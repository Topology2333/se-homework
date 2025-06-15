<template>
  <div class="app-container">
    <header>
      <div class="logo">
        <i class="el-icon-lightning logo-icon"></i>
        <h1>充电详单</h1>
      </div>
      <div class="user-info">
        <div class="user-avatar">{{ userInitial }}</div>
        <span>{{ userName }}</span>
        <el-button type="info" size="small" icon="el-icon-switch-button" @click="logout">退出</el-button>
      </div>
    </header>

    <div class="content">
      <h2 class="page-title">
        <i class="el-icon-document"></i>
        充电详单记录
      </h2>

      <div class="stats-container">
        <div class="stat-card">
          <div class="stat-title">总充电次数</div>
          <div class="stat-value">{{ stats.total_count }} 次</div>
        </div>
        <div class="stat-card green">
          <div class="stat-title">总充电量</div>
          <div class="stat-value">{{ stats.total_amount.toFixed(1) }} 度</div>
        </div>
        <div class="stat-card orange">
          <div class="stat-title">总消费金额</div>
          <div class="stat-value">¥{{ stats.total_fee.toFixed(2) }}</div>
        </div>
        <div class="stat-card purple">
          <div class="stat-title">平均充电时长</div>
          <div class="stat-value">{{ stats.avg_duration.toFixed(1) }} 小时</div>
        </div>
      </div>

      <div class="table-container">
        <div class="action-bar">
          <div class="search-container">
            <el-input 
              v-model="searchParams.pile_number" 
              placeholder="搜索充电桩编号..." 
              clearable 
              style="width: 250px;"
            ></el-input>
            <el-date-picker
              v-model="searchParams.date_range"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              value-format="YYYY-MM-DD"
              @change="handleSearch"
            >
            </el-date-picker>
            <el-select 
              v-model="searchParams.mode" 
              placeholder="充电模式" 
              clearable 
              style="width: 120px;"
              @change="handleSearch"
            >
              <el-option label="快充" value="fast"></el-option>
              <el-option label="慢充" value="slow"></el-option>
            </el-select>
            <el-button type="primary" icon="el-icon-search" @click="handleSearch">搜索</el-button>
          </div>
        </div>

        <el-table 
          v-loading="loading"
          :data="filteredRecords" 
          class="charging-table" 
          style="width: 100%"
        >
          <el-table-column prop="id" label="详单ID" width="120"></el-table-column>
          <el-table-column prop="pile_number" label="充电桩编号" width="150"></el-table-column>
          <el-table-column label="充电模式" width="120">
            <template #default="scope">
              <span :class="['mode-tag', scope.row.mode === '快充' ? 'mode-fast' : 'mode-slow']">
                {{ scope.row.mode }}
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="start_time" label="开始时间" width="180"></el-table-column>
          <el-table-column prop="end_time" label="结束时间" width="180"></el-table-column>
          <el-table-column prop="duration" label="充电时长" width="120" align="center">
            <template #default="scope">
              {{ scope.row.duration }} 小时
            </template>
          </el-table-column>
          <el-table-column prop="amount" label="充电量(度)" width="120" align="center"></el-table-column>
          <el-table-column prop="charging_fee" label="充电费用" width="120" align="center">
            <template #default="scope">
              ¥{{ safeToFixed(scope.row.charging_fee) }}
            </template>
          </el-table-column>
          <el-table-column prop="service_fee" label="服务费" width="120" align="center">
            <template #default="scope">
              ¥{{ safeToFixed(scope.row.service_fee) }}
            </template>
          </el-table-column>
          <el-table-column label="总费用" width="120" align="center">
            <template #default="scope">
              <strong>¥{{ safeToFixed(scope.row.total_fee) }}</strong>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="120" align="center">
            <template #default="scope">
              <span :class="['status-badge', scope.row.status === '已完成' ? 'status-completed' : 'status-in-progress']">
                {{ scope.row.status }}
              </span>
            </template>
          </el-table-column>
        </el-table>

        <div class="pagination">
          <el-pagination
            background
            layout="prev, pager, next"
            :total="stats.total_count"
            :page-size="pageSize"
            :current-page="currentPage"
            @current-change="handlePageChange"
          >
          </el-pagination>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted } from 'vue';
import axios from 'axios';
import { ElMessage } from 'element-plus';
import { useRouter } from 'vue-router';

export default {
  name: 'DetailBills',
  setup() {
    const router = useRouter();
    const currentPage = ref(1);
    const pageSize = ref(10);
    const loading = ref(false);
    
    // 从本地存储获取用户信息
    const user = JSON.parse(localStorage.getItem('user'));
    const userName = ref(user ? user.username : '');
    const userInitial = ref(user ? user.username.charAt(0) : '');
    
    const searchParams = ref({
      pile_number: '',
      date_range: [],
      mode: ''
    });
    
    const stats = ref({
      total_count: 0,
      total_amount: 0,
      total_fee: 0,
      avg_duration: 0
    });
    
    const allChargingRecords = ref([]);
    
    // 安全转换数字为固定小数位
    const safeToFixed = (value, decimals = 2) => {
      const num = parseFloat(value);
      return isNaN(num) ? '0.00' : num.toFixed(decimals);
    };

    // 计算筛选后的记录
    const filteredRecords = computed(() => {
      // 先按开始时间从晚到早排序
      let result = [...allChargingRecords.value].sort((a, b) => new Date(b.start_time) - new Date(a.start_time));
      
      // 按充电桩编号筛选
      if (searchParams.value.pile_number) {
         result = result.filter(record =>
         record.pile_number === searchParams.value.pile_number
        );
      }
      
      // 按充电模式筛选
      if (searchParams.value.mode) {
        const targetMode = searchParams.value.mode === 'fast' ? '快充' : '慢充';
        result = result.filter(record => record.mode === targetMode);
      }
      
      // 按日期范围筛选
      if (searchParams.value.date_range && searchParams.value.date_range.length === 2) {
        const startDate = new Date(searchParams.value.date_range[0]);
        const endDate = new Date(searchParams.value.date_range[1]);
        endDate.setDate(endDate.getDate() + 1); // 包括结束日期当天
        
        result = result.filter(record => {
          const recordDate = new Date(record.start_time);
          return recordDate >= startDate && recordDate < endDate;
        });
      }
      
      // 更新统计信息
      updateStats(result);
      
      // 分页处理
      const startIndex = (currentPage.value - 1) * pageSize.value;
      return result.slice(startIndex, startIndex + pageSize.value);
    });

    // 更新统计信息
    const updateStats = (records) => {
      const totalCount = records.length;
      if (totalCount > 0) {
        const totalAmount = records.reduce((sum, r) => sum + parseFloat(r.amount || 0), 0);
        const totalFee = records.reduce((sum, r) => sum + parseFloat(r.total_fee || 0), 0);
        const totalDuration = records.reduce((sum, r) => sum + parseFloat(r.duration || 0), 0);
        
        stats.value = {
          total_count: totalCount,
          total_amount: totalAmount,
          total_fee: totalFee,
          avg_duration: totalDuration / totalCount
        };
      } else {
        stats.value = {
          total_count: 0,
          total_amount: 0,
          total_fee: 0,
          avg_duration: 0
        };
      }
    };

    // 从后端API获取充电记录
    const fetchChargingRecords = async () => {
      try {
        loading.value = true;
        
        if (!user || !user.id) {
          ElMessage.error('用户未登录');
          router.push('/home');
          return;
        }
        
        const response = await axios.get(`http://localhost:8080/users/${user.id}/charging_records`);
        
        allChargingRecords.value = response.data.map(record => ({
          id: record.id,
          pile_number: record.pile_id,
          mode: record.mode === "Fast" ? "快充" : "慢充",
          start_time: new Date(record.start_time).toLocaleString(),
          end_time: new Date(record.end_time).toLocaleString(),
          duration: record.charging_time.toFixed(2),
          amount: record.charging_amount,
          charging_fee: record.charging_fee,
          service_fee: record.service_fee,
          total_fee: record.total_fee,
          status: record.status === 'completed' ? '已完成' : '进行中'
        }));
        
        // 初始化统计数据
        updateStats(allChargingRecords.value);
        
      } catch (error) {
        console.error('获取充电记录失败:', error);
        ElMessage.error('获取充电记录失败，请稍后重试');
      } finally {
        loading.value = false;
      }
    };
    
    // 处理搜索操作
    const handleSearch = () => {
      currentPage.value = 1; // 重置到第一页
    };
    
    const handlePageChange = (page) => {
      currentPage.value = page;
    };
    
    // 退出登录
    const logout = () => {
      localStorage.removeItem('user');
      router.push('/');
    };
    
    onMounted(() => {
      fetchChargingRecords();
    });
    
    return {
      currentPage,
      filteredRecords,
      stats,
      searchParams,
      loading,
      pageSize,
      handlePageChange,
      handleSearch,
      logout,
      safeToFixed,
      userName,
      userInitial
    };
  }
};
</script>

<style>
.app-container {
  max-width: 1200px;
  margin: 0 auto;
  background-color: white;
  border-radius: 15px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

/* 保留其他样式不变 */
header {
  background: linear-gradient(to right, #1a2980, #26d0ce);
  color: white;
  padding: 15px 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.logo {
  display: flex;
  align-items: center;
  gap: 15px;
}

.logo h1 {
  font-size: 24px;
  font-weight: 600;
}

.logo-icon {
  font-size: 28px;
}

nav ul {
  display: flex;
  list-style: none;
  gap: 25px;
}

nav a {
  color: rgba(255, 255, 255, 0.9);
  text-decoration: none;
  font-weight: 500;
  font-size: 16px;
  transition: all 0.3s ease;
  padding: 8px 15px;
  border-radius: 8px;
}

nav a:hover, nav a.router-link-active {
  background-color: rgba(255, 255, 255, 0.15);
  color: white;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background-color: #3498db;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: bold;
}

.content {
  padding: 30px;
}

.page-title {
  font-size: 28px;
  color: #2c3e50;
  margin-bottom: 30px;
  padding-bottom: 15px;
  border-bottom: 2px solid #eee;
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-title i {
  font-size: 26px;
  color: #3498db;
}

.stats-container {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.stat-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
  border-left: 4px solid #3498db;
}

.stat-card.green {
  border-left-color: #2ecc71;
}

.stat-card.orange {
  border-left-color: #f39c12;
}

.stat-card.purple {
  border-left-color: #9b59b6;
}

.stat-title {
  font-size: 16px;
  color: #7f8c8d;
  margin-bottom: 10px;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #2c3e50;
}

.table-container {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.action-bar {
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 15px;
}

.search-container {
  display: flex;
  gap: 15px;
}

.action-buttons {
  display: flex;
  gap: 10px;
}

.charging-table {
  width: 100%;
  border-collapse: collapse;
}

.charging-table th {
  background-color: #f8f9fa;
  color: #2c3e50;
  font-weight: 600;
  text-align: left;
  padding: 15px;
  border-bottom: 2px solid #eee;
}

.charging-table td {
  padding: 15px;
  border-bottom: 1px solid #eee;
  color: #34495e;
}

.charging-table tr:hover {
  background-color: #f8f9fa;
}

.status-badge {
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
}

.status-completed {
  background-color: #e8f6f0;
  color: #2ecc71;
}

.status-in-progress {
  background-color: #fef9e7;
  color: #f39c12;
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 30px;
}

.no-data {
  text-align: center;
  padding: 40px;
  color: #7f8c8d;
}

.no-data i {
  font-size: 60px;
  margin-bottom: 20px;
  color: #bdc3c7;
}

.no-data p {
  font-size: 18px;
}

footer {
  display: none;
}

.mode-tag {
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.mode-fast {
  background-color: #e3f2fd;
  color: #1976d2;
}

.mode-slow {
  background-color: #f3e5f5;
  color: #9c27b0;
}

@media (max-width: 768px) {
  .action-bar {
    flex-direction: column;
  }

  .search-container {
    width: 100%;
  }

  .action-buttons {
    width: 100%;
    justify-content: center;
  }

  nav ul {
    display: none;
  }
}
</style>