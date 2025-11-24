import { useState, useEffect } from 'react';
import { api } from '../services/apiClient';

export const useAPIData = (period = '7d', autoRefresh = false) => {
  const [data, setData] = useState([]);
  const [stats, setStats] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const [usageResponse, statsResponse] = await Promise.all([
        api.getUsage({ period }),
        api.getStats({ period })
      ]);
      
      setData(usageResponse.data || []);
      setStats(statsResponse.data);
    } catch (err) {
      setError(err.message || 'Failed to fetch data');
      console.error('Error fetching API data:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    
    // Auto-refresh every 30 seconds if enabled
    if (autoRefresh) {
      const interval = setInterval(fetchData, 30000);
      return () => clearInterval(interval);
    }
  }, [period, autoRefresh]);

  return { 
    data, 
    stats, 
    loading, 
    error, 
    refetch: fetchData 
  };
};