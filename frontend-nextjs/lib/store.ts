import { create } from 'zustand'

// 用户可访问的连接信息（来自后端）
interface UserConnection {
  user_id: number
  username: string
  tenant_id: number
  tenant_name: string
  database_id: number
  connection_name: string
  db_host: string
  db_port: number
  db_name: string
  is_primary: boolean
  user_role: string
}

interface Database {
  id: string
  name: string
  host: string
  port: number
  database: string
  description?: string
}

interface UserInfo {
  id: number
  username: string
  email: string
  role: string  // user, admin 等
  is_superadmin?: boolean  // 超级管理员标识
  created_at: string
}

interface AppState {
  // 用户信息
  currentUser: UserInfo | null
  setCurrentUser: (user: UserInfo | null) => void
  
  // 新的多租户连接管理
  currentConnection: UserConnection | null
  setCurrentConnection: (conn: UserConnection | null) => void
  userConnections: UserConnection[]
  setUserConnections: (conns: UserConnection[]) => void
  
  // 旧的数据库选择（保留兼容性）
  currentDatabase: Database | null
  setCurrentDatabase: (db: Database | null) => void
  currentSchema: string
  setCurrentSchema: (schema: string) => void
  databases: Database[]
  setDatabases: (dbs: Database[]) => void
  addDatabase: (db: Database) => void
  removeDatabase: (id: string) => void
}

export const useAppStore = create<AppState>()((set) => ({
  // 用户信息
  currentUser: typeof window !== 'undefined' && localStorage.getItem('current_user')
    ? JSON.parse(localStorage.getItem('current_user')!)
    : null,
  setCurrentUser: (user) => {
    set({ currentUser: user })
    if (typeof window !== 'undefined') {
      if (user) {
        localStorage.setItem('current_user', JSON.stringify(user))
      } else {
        localStorage.removeItem('current_user')
      }
    }
  },
  
  // 新的多租户支持
  currentConnection: typeof window !== 'undefined' && localStorage.getItem('current_connection')
    ? JSON.parse(localStorage.getItem('current_connection')!)
    : null,
  setCurrentConnection: (conn) => {
    set({ currentConnection: conn })
    if (typeof window !== 'undefined') {
      if (conn) {
        localStorage.setItem('current_connection', JSON.stringify(conn))
      } else {
        localStorage.removeItem('current_connection')
      }
    }
  },
  userConnections: typeof window !== 'undefined' && localStorage.getItem('user_connections')
    ? JSON.parse(localStorage.getItem('user_connections')!)
    : [],
  setUserConnections: (conns) => {
    set({ userConnections: conns })
    if (typeof window !== 'undefined') {
      localStorage.setItem('user_connections', JSON.stringify(conns))
    }
  },
  
  // 旧的实现（保留兼容性）
  currentDatabase: typeof window !== 'undefined' && localStorage.getItem('current_database')
    ? JSON.parse(localStorage.getItem('current_database')!)
    : {
        id: 'default',
        name: 'Default Project',
        host: 'localhost',
        port: 5432,
        database: 'crestrail',
        description: '默认数据库连接',
      },
  setCurrentDatabase: (db) => {
    set({ currentDatabase: db })
    if (typeof window !== 'undefined' && db) {
      localStorage.setItem('current_database', JSON.stringify(db))
    }
  },

  currentSchema: typeof window !== 'undefined' && localStorage.getItem('current_schema')
    ? localStorage.getItem('current_schema')!
    : 'public',
  setCurrentSchema: (schema) => {
    set({ currentSchema: schema })
    if (typeof window !== 'undefined') {
      localStorage.setItem('current_schema', schema)
    }
  },

  databases: typeof window !== 'undefined' && localStorage.getItem('databases')
    ? JSON.parse(localStorage.getItem('databases')!)
    : [
        {
          id: 'default',
          name: 'Default Project',
          host: 'localhost',
          port: 5432,
          database: 'crestrail',
          description: '默认数据库连接',
        },
      ],
  setDatabases: (dbs) => {
    set({ databases: dbs })
    if (typeof window !== 'undefined') {
      localStorage.setItem('databases', JSON.stringify(dbs))
    }
  },
  addDatabase: (db) =>
    set((state) => {
      const newDatabases = [...state.databases, db]
      if (typeof window !== 'undefined') {
        localStorage.setItem('databases', JSON.stringify(newDatabases))
      }
      return { databases: newDatabases }
    }),
  removeDatabase: (id) =>
    set((state) => {
      const newDatabases = state.databases.filter((db) => db.id !== id)
      if (typeof window !== 'undefined') {
        localStorage.setItem('databases', JSON.stringify(newDatabases))
      }
      return { databases: newDatabases }
    }),
}))
