import matplotlib.pyplot as plt
import matplotlib.animation as animation
import numpy as np
import pandas as pd

fig, ax = plt.subplots()

def animate(frame):
    ax.clear()

    data = pd.read_csv("data.csv")

    groups = data['Program']
    packets_sent = data['Packets_Sent']
    packets_lost = data['Packets_Received']
    packets_ignored = data['Packets_Ignored']

    x = np.arange(len(groups))
    width = 0.25

    ax.bar(x - width, packets_sent, width, label='Packets Sent', color='green')
    ax.bar(x, packets_lost, width, label='Packets Received', color='blue')
    ax.bar(x + width, packets_ignored, width, label='Packets Ignored', color='red')

    ax.set_xlabel('Programs')
    ax.set_ylabel('Packets')
    ax.set_title('Packets')
    ax.set_xticks(x)
    ax.set_xticklabels(groups)
    ax.legend()

    plt.tight_layout()

ani = animation.FuncAnimation(fig, animate, interval=1000)
plt.show()
