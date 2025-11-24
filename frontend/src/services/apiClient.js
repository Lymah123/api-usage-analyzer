import axios from 'axios';
import toast from 'react-hot-toast';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  }
});

// Request interceptor
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor
apiClient.interceptors.response.use(
  (response) => response.data,
  (error) => {
    const message = error.response?.data?.error || error.message || 'An error occurred';
    
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
      toast.error('Session expired. Please login again.');
    } else {
      toast.error(message);
    }
    
    return Promise.reject(error.response?.data || error);
  }
);

// API methods
export const api = {
  // Auth
  login: (credentials) => apiClient.post('/auth/login', credentials),
  register: (userData) => apiClient.post('/auth/register', userData),
  logout: () => apiClient.post('/auth/logout'),
  getCurrentUser: () => apiClient.get('/auth/me'),
  
  // Usage
  getUsage: (params) => apiClient.get('/usage', { params }),
  recordUsage: (data) => apiClient.post('/usage', data),
  getStats: (params) => apiClient.get('/usage/stats', { params }),
  exportUsage: (params) => apiClient.get('/usage/export', { 
    params, 
    responseType: 'blob' 
  }),
  
  // Predictions
  getPredictions: () => apiClient.get('/predictions'),
  generatePrediction: (data) => apiClient.post('/predictions/generate', data),
  
  // API Keys
  getAPIKeys: () => apiClient.get('/api-keys'),
  createAPIKey: (data) => apiClient.post('/api-keys', data),
  updateAPIKey: (id, data) => apiClient.put(`/api-keys/${id}`, data),
  deleteAPIKey: (id) => apiClient.delete(`/api-keys/${id}`),
  
  // Analytics
  getAnalytics: (params) => apiClient.get('/analytics/overview', { params }),
  getAnomalies: () => apiClient.get('/analytics/anomalies'),
};

export default apiClient;