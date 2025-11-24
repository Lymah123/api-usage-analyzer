import { useState, useEffect } from 'react';

export function usePredictions() {
  const [predictions, setPredictions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    setLoading(true);
    setError(null);

    fetch('/api/v1/predictions', {
      credentials: 'include', // if you need cookies/auth
    })
      .then((res) => {
        if (!res.ok) throw new Error('Failed to fetch predictions');
        return res.json();
      })
      .then((data) => {
        setPredictions(data);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message || 'Unknown error');
        setLoading(false);
      });
  }, []);

  return { predictions, loading, error };
}