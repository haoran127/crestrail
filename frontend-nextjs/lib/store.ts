import { create } from 'zustand'

interface Database {
  id: string
  name: string
  host: string
  port: number
  database: string
  description?: string
}

interface AppState {
  // 当前选中的数据库连接
  currentDatabase: Database | null
  setCurrentDatabase: (db: Database | null) => void

  // 当前选中的 schema
  currentSchema: string
  setCurrentSchema: (schema: string) => void

  // 数据库连接列表
  databases: Database[]
  setDatabases: (dbs: Database[]) => void
  addDatabase: (db: Database) => void
  removeDatabase: (id: string) => void
}

export const useAppStore = create<AppState>()((set) => ({
  currentDatabase: {
    id: 'default',
    name: 'Default Project',
    host: 'localhost',
    port: 5432,
    database: 'crestrail',
    description: '默认数据库连接',
  },
  setCurrentDatabase: (db) => {
    set({ currentDatabase: db })
    // 切换数据库时保存到 localStorage
    if (typeof window !== 'undefined') {
      localStorage.setItem('current_database', JSON.stringify(db))
    }
  },

  currentSchema: 'public',
  setCurrentSchema: (schema) => {
    set({ currentSchema: schema })
    // 保存到 localStorage
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

