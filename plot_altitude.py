import json
import matplotlib.pyplot as plt
import datetime
import numpy as np

def moving_average(data, window_size):
    return np.convolve(data, np.ones(window_size)/window_size, mode='valid')

# Initialize lists to store data
timestamps = []
current_altitudes = []
zero_altitudes = []
speeds = []
speed_alarms = []

# Read and parse the JSON data
with open('flight_log.json', 'r') as file:
    for line in file:
        data = json.loads(line)
        timestamp = datetime.datetime.strptime(data['timestamp'], '%Y-%m-%d %H:%M:%S.%f')
        timestamps.append(timestamp)
        current_altitudes.append(int(data['message']['Altitude']['current']))
        zero_altitudes.append(int(data['message']['Altitude']['zero']))
        speeds.append(float(data['message']['Flight']['speed']))
        speed_alarms.append(int(data['message']['Flight']['speedAlarm']))

# Smoothing parameters
window_size = 5  # Adjust this value to change smoothing intensity

# Apply smoothing
smoothed_current = moving_average(current_altitudes, window_size)
smoothed_zero = moving_average(zero_altitudes, window_size)
smoothed_speeds = moving_average(speeds, window_size)
timestamps_smoothed = timestamps[window_size-1:]

# Create subplots
fig, (ax1, ax2, ax3, ax4) = plt.subplots(4, 1, figsize=(12, 12))

# Plot 1: Altitude (Current and Zero Reference)
ax1.plot(timestamps_smoothed, smoothed_current, 'b-', label='Current Altitude (Smoothed)')
ax1.plot(timestamps_smoothed, smoothed_zero, 'r-', label='Zero Reference (Smoothed)')
ax1.plot(timestamps, current_altitudes, 'b-', alpha=0.2, label='Raw Current Altitude')
ax1.plot(timestamps, zero_altitudes, 'r-', alpha=0.2, label='Raw Zero Reference')
ax1.set_title('Altitude (Current and Zero Reference)')
ax1.legend()
ax1.grid(True)

# Plot 2: Relative Altitude
relative_altitude = [c - z for c, z in zip(current_altitudes, zero_altitudes)]
smoothed_relative = moving_average(relative_altitude, window_size)
ax2.plot(timestamps_smoothed, smoothed_relative, 'g-', label='Smoothed')
ax2.plot(timestamps, relative_altitude, 'g-', alpha=0.2, label='Raw')
ax2.set_title('Relative Altitude')
ax2.legend()
ax2.grid(True)

# Plot 3: Speed
ax3.plot(timestamps_smoothed, smoothed_speeds, 'b-', label='Smoothed')
ax3.plot(timestamps, speeds, 'b-', alpha=0.2, label='Raw')
ax3.set_title('Speed')
ax3.legend()
ax3.grid(True)

# Plot 4: Speed Alarms
alarm_times = [t for t, a in zip(timestamps, speed_alarms) if a == 1]
ax4.scatter(alarm_times, [1] * len(alarm_times), color='red', marker='o')
ax4.set_title('Speed Alarms')
ax4.set_ylim(0, 2)
ax4.grid(True)

plt.tight_layout()
plt.savefig('flight_data_plot_python.png')
plt.show()
