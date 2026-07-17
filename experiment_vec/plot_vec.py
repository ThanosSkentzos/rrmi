# %%
import matplotlib.pyplot as plt
import pandas as pd
import os

os.makedirs("plots",exist_ok=True)
# print("Reading data")
folder = "./experiment_vec"
# folder = "."
eth_rust = pd.read_csv(f"{folder}/results_vec_rrmi.csv", index_col=False)
eth_rust_grpc = pd.read_csv(f"{folder}/results_vec_grpc.csv", index_col=False)
ib_rust = pd.read_csv(f"{folder}/results_vec_rrmi_ib.csv", index_col=False)
ib_rust_grpc = pd.read_csv(f"{folder}/results_vec_grpc_ib.csv", index_col=False)

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
ib_text = " (ib)"
# eth_rust_dfs = {obj_type: sub_df.reset_index() for obj_type, sub_df in eth_rust.groupby("Type") if obj_type != "Type"}
# eth_java_dfs = {obj_type+java_text: sub_df.reset_index() for obj_type, sub_df in eth_java.groupby("Type") if obj_type != "Type"}
# ib_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib.groupby("Type") if obj_type != "Type"}
# ib_java_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib_java.groupby("Type") if obj_type != "Type"}

#%%
cols = ["Latency","Throughput","Size"]
# eth_rust_groups = eth_rust.groupby(["Type","NClients"])

fig_lat, ax_lat = plt.subplots(figsize=(12, 8))
fig_thr, ax_thr = plt.subplots(figsize=(12, 8))
# fig_lat_ib, ax_lat_ib = plt.subplots(figsize=(12, 8))
# fig_thr_ib, ax_thr_ib = plt.subplots(figsize=(12, 8))
eth_ax = [ax_lat,ax_thr]
# ib_ax = [ax_lat_ib,ax_thr_ib]

b = "#378ADD"
c = "#37C4DD"
o = "#D87930"
y = "#E0BC1E"
r = "#D84630"
g = "#197C2A"
pu = "#7E21A3"
pi = "#A321A3"

alpha=0.2

type_keys = ["rrmi","grpc","rrmi"+ib_text,"grpc"+ib_text]
linestyle_values = ['solid','solid','dotted','dotted']
linestyles = dict(zip(type_keys,linestyle_values))
marker_values = ['^','s'] *2
markers = dict(zip(type_keys,marker_values))

x_key = "Size"
l_list=[]
t_list=[]
ls_list=[]
ts_list=[]
def plot(df,cols,ax,color,linestyles,markers,framework):
    sub = df[cols]
    sub = sub.sort_values(x_key)
    mean = sub.groupby([x_key]).mean()
    std = sub.groupby([x_key]).std()
    num_clients = mean.index.values

    linestyle = linestyles[framework]
    marker = markers[framework]
    # color = colors[text]
    label = framework

    for c,a in zip(mean.columns,ax):
        m = mean[c]
        s = std[c]

        a.plot(num_clients,m,label=label,marker=marker,linestyle=linestyle,c=color,markerfacecolor='white')
        a.fill_between(num_clients,m-s,m+s,alpha=alpha,color=color)
        # a.plot(num_clients,m,c=color,markerfacecolor='white')

        a.set_xlabel('Number of Bytes')
        a.legend(
            title = " "*5+"This work"+" "*10+"gRPC"+" "*10,
            ncol=2,shadow=True, 
            # bbox_to_anchor = (0.1,0.12,0.4,.47),
            loc="center right"
            )
        a.grid(True, alpha=0.3)
        a.set_yscale("log")
        a.set_xscale("log",base=2,)

        type = m.name
        m.name = framework + "_"
        s.name = framework + "_"
        if type=="Latency":
            l_list.append(m)
            ls_list.append(s)
        else:
            t_list.append(m)
            ts_list.append(s)

plot(eth_rust,cols,eth_ax,o,linestyles,markers,"rrmi")
plot(ib_rust,cols,eth_ax,o,linestyles,markers,"rrmi"+ib_text)

plot(eth_rust_grpc,cols,eth_ax,g,linestyles,markers,"grpc")
plot(ib_rust_grpc,cols,eth_ax,g,linestyles,markers,"grpc"+ib_text)

latency = pd.concat(l_list,axis=1)
throughput = pd.concat(t_list,axis=1)
print(latency)
print(throughput)

eth_ax[0].axvline(2**22,color=g,linestyle="dashed")
eth_ax[1].axvline(2**22,color=g,linestyle='dashed')

x_axis = "Number of Bytes"
y_axis = "Average Latency"
ax_lat.set_ylabel(f'{y_axis} (μsec)')
ax_lat.set_title(f'{y_axis} vs {x_axis}')

y_axis_thr="Average Throughput"
ax_thr.set_ylabel(f'{y_axis_thr} (bps)')
ax_thr.set_title(f'{y_axis_thr} vs {x_axis}')

fig_lat.savefig("plots/vec_latency.png")
fig_thr.savefig("plots/vec_throughput.png")