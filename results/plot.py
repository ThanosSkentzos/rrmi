# %%
import matplotlib.pyplot as plt
import pandas as pd
import os

os.makedirs("plots",exist_ok=True)
# print("Reading data")
eth_rust = pd.read_csv("results/results_rust.csv", index_col=False)
eth_rust_grpc = pd.read_csv("results/results_rust_grpc.csv", index_col=False)
eth_java = pd.read_csv("results/results_java.csv", index_col=False)
ib_rust = pd.read_csv("results/results_rust_ib.csv", index_col=False)
ib_java = pd.read_csv("results/results_java_ib.csv", index_col=False)
ib_rust_grpc = pd.read_csv("results/results_rust_grpc_ib.csv", index_col=False)

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
fig_lat_ib, ax_lat_ib = plt.subplots(figsize=(12, 8))
fig_thr_ib, ax_thr_ib = plt.subplots(figsize=(12, 8))

b = "#378ADD"
c = "#37C4DD"
o = "#D87930"
y = "#E0BC1E"
r = "#D84630"
g = "#197C2A"
pu = "#7E21A3"
pi = "#A321A3"

eth_ax = [ax_lat,ax_thr]
ib_ax = [ax_lat_ib,ax_thr_ib]
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
            a.legend(
                title = "This work"+" "*25+"Java RMI"+" "*35+"gRPC"+" "*10,
                ncol=3,shadow=True, 
                bbox_to_anchor = (0.1,0.12,0.4,.47),
                loc="center left"
                )
            a.grid(True, alpha=0.3)
            a.set_yscale("log")

plot(eth_rust,cols,eth_ax,o,linestyles,markers,"")
plot(eth_java,cols,eth_ax,b,linestyles,markers,"")
plot(eth_rust_grpc,cols,eth_ax,g,linestyles,markers,"")

plot(ib_rust,cols,ib_ax,o,linestyles,markers,"")
plot(ib_java,cols,ib_ax,b,linestyles,markers,"")
plot(ib_rust_grpc,cols,ib_ax,g,linestyles,markers,"")

ax_lat.set_ylabel('Average Latency (μsec)')
ax_lat.set_title('Average Latency vs Number of Clients - Using Ethernet')
ax_lat_ib.set_ylabel('Average Latency (μsec)')
ax_lat_ib.set_title('Average Latency vs Number of Clients - Using Infiniband')

ax_thr.set_ylabel('Average Throughput (bps)')
ax_thr.set_title('Average Throughput vs Number of Clients - Using Ethernet')
ax_thr_ib.set_ylabel('Average Throughput (bps)')
ax_thr_ib.set_title('Average Throughput vs Number of Clients - Using Infiniband')

fig_lat.savefig("plots/latency.png")
fig_thr.savefig("plots/throughput.png")
fig_lat_ib.savefig("plots/latency_ib.png")
fig_thr_ib.savefig("plots/throughput_ib.png")

# %%
