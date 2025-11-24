import React, { useState } from 'react';
import toast from 'react-hot-toast';

export default function Settings() {
  const [form, setForm] = useState({
    name: '',
    organization: '',
    password: '',
    confirmPassword: '',
  });
  const [loading, setLoading] = useState(false);

  const handleChange = (e) => {
    setForm({ ...form, [e.target.name]: e.target.value });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (form.password && form.password !== form.confirmPassword) {
      toast.error('Passwords do not match');
      return;
    }
    setLoading(true);
    toast.dismiss();

    try {
      const res = await fetch('/api/v1/user/settings', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          name: form.name,
          organization: form.organization,
          password: form.password || undefined,
        }),
      });

      const data = await res.json();

      if (!res.ok || !data.success) {
        throw new Error(data.message || 'Update failed');
      }

      toast.success('Settings updated!');
      setForm({ ...form, password: '', confirmPassword: '' });
    } catch (err) {
      toast.error(err.message || 'Update failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-lg mx-auto mt-10 bg-white p-8 rounded shadow">
      <h2 className="text-2xl font-bold mb-6">Account Settings</h2>
      <form onSubmit={handleSubmit}>
        <div className="mb-4">
          <label className="block mb-1 text-gray-700">Name</label>
          <input
            type="text"
            name="name"
            className="w-full border px-3 py-2 rounded"
            value={form.name}
            onChange={handleChange}
            placeholder="Your name"
          />
        </div>
        <div className="mb-4">
          <label className="block mb-1 text-gray-700">Organization</label>
          <input
            type="text"
            name="organization"
            className="w-full border px-3 py-2 rounded"
            value={form.organization}
            onChange={handleChange}
            placeholder="Your organization"
          />
        </div>
        <div className="mb-4">
          <label className="block mb-1 text-gray-700">New Password</label>
          <input
            type="password"
            name="password"
            className="w-full border px-3 py-2 rounded"
            value={form.password}
            onChange={handleChange}
            placeholder="Leave blank to keep current"
          />
        </div>
        <div className="mb-6">
          <label className="block mb-1 text-gray-700">Confirm New Password</label>
          <input
            type="password"
            name="confirmPassword"
            className="w-full border px-3 py-2 rounded"
            value={form.confirmPassword}
            onChange={handleChange}
            placeholder="Confirm new password"
          />
        </div>
        <button
          type="submit"
          className="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700"
          disabled={loading}
        >
          {loading ? 'Saving...' : 'Save Changes'}
        </button>
      </form>
    </div>
  );
}