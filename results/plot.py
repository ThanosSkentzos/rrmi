# %%
import matplotlib.pyplot as plt
import pandas as pd
import os

os.makedirs("plots",exist_ok=True)
# print("Reading data")
eth_rust = pd.read_csv("results_rust.csv", index_col=False)
eth_java = pd.read_csv("results_java.csv", index_col=False)
ib_rust = pd.read_csv("results_rust_ib.csv", index_col=False)
ib_java = pd.read_csv("results_java_ib.csv", index_col=False)

# df.columns = [
#     "Number of Clients",
#     "Total Requests",
#     "Total Aggregated Time",
#     "Roundtrip",
#     "Latency",
#     "Throughput",
# ]
# %%
rrmi_text = "This work - "
java_text = "Java RMI - "
ib_text = " (ib)"
# eth_rust_dfs = {obj_type: sub_df.reset_index() for obj_type, sub_df in eth_rust.groupby("Type") if obj_type != "Type"}
# eth_java_dfs = {obj_type+java_text: sub_df.reset_index() for obj_type, sub_df in eth_java.groupby("Type") if obj_type != "Type"}
# ib_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib.groupby("Type") if obj_type != "Type"}
# ib_java_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib_java.groupby("Type") if obj_type != "Type"}

#%%
cols = ["NClients","Latency","Throughput"]
# eth_rust_groups = eth_rust.groupby(["Type","NClients"])

fig_lat, ax_lat = plt.subplots(figsize=(12, 8))
fig_thr, ax_thr = plt.subplots(figsize=(12, 8))
b = "#378ADD"
c = "#37C4DD"
o = "#D87930"
y = "#E0BC1E"
r = "#D84630"
g = "#197C2A"
pu = "#7E21A3"
pi = "#A321A3"

eth_ax = [ax_lat,ax_thr]
alpha=0.2

type_keys = ["Sequence","Vector","Hashmap"]
color_values = [b,r,g]
colors = dict(zip(type_keys,color_values))
linestyle_values = ['dotted','solid','dashed']
linestyles = dict(zip(type_keys,linestyle_values))
marker_values = ['.','s','^']
markers = dict(zip(type_keys,marker_values))

def plot(df,cols,ax,color,linestyles,markers,text):
    for t, sub in df.groupby(["Type"],sort=False)[cols]:
        sub = sub.sort_values('NClients')
        mean = sub.groupby(['NClients']).mean()
        std = sub.groupby(['NClients']).std()
        num_clients = mean.index.values

        linestyle = linestyles[t[0]]
        marker = markers[t[0]]
        label = t[0] + text

        for c,a in zip(mean.columns,ax):
            m = mean[c]
            s = std[c]

            a.plot(num_clients,m,label=label,marker=marker,linestyle=linestyle,c=color,markerfacecolor='white')
            a.fill_between(num_clients,m-s,m+s,alpha=alpha,color=color)

            a.set_xlabel('Number of Clients')
            # a.legend(title="Experiment and framework")
            a.legend(title = "This work                        Java RMI",ncol=2,shadow=True)
            a.grid(True, alpha=0.3)
            a.set_yscale("log")

plot(eth_rust,cols,eth_ax,o,linestyles,markers,"")
plot(ib_rust,cols,eth_ax,y,linestyles,markers,ib_text)
plot(eth_java,cols,eth_ax,b,linestyles,markers,"")
plot(ib_java,cols,eth_ax,c,linestyles,markers,ib_text)

ax_lat.set_ylabel('Average Latency (μsec)')
ax_lat.set_title('Average Latency vs Number of Clients')

ax_thr.set_ylabel('Average Throughput (bps)')
ax_thr.set_title('Average Throughput vs Number of Clients')

fig_lat.savefig("latency.png")
fig_thr.savefig("throughput.png")

# %%
