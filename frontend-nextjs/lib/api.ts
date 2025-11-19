import axios from 'axios'

// 创建 API 客户端实例
const api = axios.create({
  // 在浏览器端直接请求后端，不通过 Next.js 代理
  baseURL: typeof window !== 'undefined' 
    ? 'http://localhost:3000' 
    : process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: false, // 不发送 cookies
})

// 请求拦截器
api.interceptors.request.use(
  (config) => {
    // 从 localStorage 获取 token
    if (typeof window !== 'undefined') {
      const token = localStorage.getItem('token')
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }
      
      // 添加当前选中的数据库连接 ID 到请求头
      const currentConnection = localStorage.getItem('current_connection')
      if (currentConnection) {
        try {
          const conn = JSON.parse(currentConnection)
          if (conn && conn.database_id) {
            config.headers['X-Database-Id'] = conn.database_id.toString()
          }
        } catch (e) {
          console.error('解析 current_connection 失败:', e)
        }
      }
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Token 过期，跳转到登录页
      if (typeof window !== 'undefined') {
        localStorage.removeItem('token')
        window.location.href = '/login'
      }
    }
    return Promise.reject(error)
  }
)

export default api

// API 方法封装
export const authAPI = {
  login: (email: string, password: string) =>
    api.post('/auth/login', { email, password }),
  
  register: (email: string, password: string) =>
    api.post('/auth/register', { email, password }),
  
  me: () => api.get('/auth/me'),
  
  changePassword: (old_password: string, new_password: string) =>
    api.post('/auth/change-password', { old_password, new_password }),
}

export const schemaAPI = {
  listSchemas: () => api.get('/api/schemas'),
  listTables: (schema: string) => api.get(`/api/schema/${schema}/tables`),
  getTableStructure: (schema: string, table: string) =>
    api.get(`/api/schema/${schema}/table/${table}/structure`),
}

// 租户管理 API
export const tenantAPI = {
  // 获取当前用户可访问的所有连接
  getMyConnections: () => api.get('/api/tenants/my-connections'),
  
  // 获取指定租户的业务 Schema
  getTenantSchemas: (tenantId: number) => api.get(`/api/tenants/${tenantId}/schemas`),
  
  // 测试数据库连接
  testConnection: (data: {
    host: string
    port: number
    database: string
    username: string
    password: string
  }) => api.post('/api/tenants/test-connection', data),
  // 创建新的数据库连接
  createConnection: (data: {
    tenant_id: number
    connection_name: string
    db_host: string
    db_port: number
    db_name: string
    db_user: string
    db_password: string
    is_primary: boolean
    max_connections?: number
    connection_timeout?: number
  }) => api.post('/api/tenants/connections', data),
  
  // 切换到指定的数据库连接
  switchConnection: (databaseId: number) => 
    api.post('/api/tenants/switch-connection', { database_id: databaseId }),
  
  // 获取连接池统计信息
  getPoolStats: () => api.get('/api/tenants/pool-stats'),
}

// 超管 API（仅超级管理员可访问）
export const adminAPI = {
  // 获取所有租户列表（使用旧的超管接口）
  listAllTenants: () => api.get('/api/admin/all-tenants'),
  
  // 创建新租户（使用新接口）
  createTenant: (data: {
    name: string
    slug: string
    contact_email?: string
  }) => api.post('/api/admin/tenants/create', data),
  
  // 获取所有用户列表
  listAllUsers: () => api.get('/api/admin/all-users'),
  
  // 将用户分配给租户
  assignUserToTenant: (userId: number, data: {
    tenant_id: number
    role: string
  }) => api.post(`/api/admin/users/${userId}/assign-tenant`, data),
  
  // 获取租户详情
  getTenantDetail: (tenantId: number) => api.get(`/api/admin/tenants/${tenantId}`),
  
  // 更新租户信息
  updateTenant: (tenantId: number, data: {
    name?: string
    status?: string
    contact_email?: string
  }) => api.patch(`/api/admin/tenants/${tenantId}`, data),
  
  // 获取系统统计信息
  getSystemStats: () => api.get('/api/admin/stats'),
}

export const tableAPI = {
  getRecords: (schema: string, table: string, params?: any) =>
    api.get(`/api/${schema}/${table}`, { params }),
  
  createRecord: (schema: string, table: string, data: any) =>
    api.post(`/api/${schema}/${table}`, data),
  
  updateRecord: (schema: string, table: string, id: number, data: any) =>
    api.put(`/api/${schema}/${table}/${id}`, data),
  
  deleteRecord: (schema: string, table: string, id: number) =>
    api.delete(`/api/${schema}/${table}/${id}`),
  
  exportCSV: (schema: string, table: string, params?: any) =>
    api.get(`/api/export/${schema}/${table}/csv`, {
      params,
      responseType: 'blob',
    }),
  
  exportJSON: (schema: string, table: string, params?: any) =>
    api.get(`/api/export/${schema}/${table}/json`, { params }),
}

export const queryAPI = {
  execute: (sql: string) => api.post('/query', { sql }),
  
  exportCSV: (sql: string) =>
    api.post('/api/export/query/csv', { sql }, { responseType: 'blob' }),
}

export const transactionAPI = {
  execute: (operations: any[]) => api.post('/transaction', { operations }),
}

export const monitorAPI = {
  getDatabaseStats: () => api.get('/api/monitor/stats'),
  
  getTableSizes: (limit?: number) =>
    api.get('/api/monitor/table-sizes', { params: { limit } }),
  
  getSlowQueries: (duration_ms?: number) =>
    api.get('/api/monitor/slow-queries', { params: { duration_ms } }),
  
  getActiveConnections: () => api.get('/api/monitor/connections'),
}

