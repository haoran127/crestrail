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

