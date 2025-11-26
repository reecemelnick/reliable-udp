import matplotlib.pyplot as plt
import matplotlib.animation as animation
import numpy as np
import pandas as pd

fig, ax = plt.subplots()
fig.canvas.manager.set_window_title("Packet Monitor")

def animate(frame):
    ax.clear()

    data = pd.read_csv("data.csv")

    groups = data['program']
    packets_sent = data['packets_sent']
    packets_lost = data['packets_received']
    packets_ignored = data['packets_ignored']
    packets_dropped = data['packets_dropped']

    x = np.arange(len(groups))
    width = 0.2

    ax.bar(x - 1.5*width, packets_sent,     width, label='Packets Sent',     color='green')
    ax.bar(x - 0.5*width, packets_lost,     width, label='Packets Received',     color='blue')
    ax.bar(x + 0.5*width, packets_ignored,  width, label='Packets Ignored',  color='red')
    ax.bar(x + 1.5*width, packets_dropped,  width, label='Packets Dropped',  color='purple')

    ax.set_xlabel('Programs')
    ax.set_ylabel('Packets')
    ax.set_title('Packets')
    ax.set_xticks(x)
    ax.set_xticklabels(groups)
    ax.legend()

    plt.tight_layout()

ani = animation.FuncAnimation(fig, animate, interval=1000)
plt.show()
