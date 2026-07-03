# %%
import matplotlib.pyplot as plt
import pandas as pd
import os

os.makedirs("plots",exist_ok=True)
print("Reading data")
eth_rust = pd.read_csv("results_rust.csv", index_col=False)
eth_java = pd.read_csv("results_java.csv", index_col=False)
# ib = pd.read_csv("results_ib.csv", index_col=False)
# ib_java = pd.read_csv("results_java_ib.csv", index_col=False)

# df.columns = [
#     "Number of Clients",
#     "Total Requests",
#     "Total Aggregated Time",
#     "Roundtrip",
#     "Latency",
#     "Throughput",
# ]
# %%
java_text = " Java"
ib_text = " (infiniband)"
eth_rust_dfs = {obj_type: sub_df.reset_index() for obj_type, sub_df in eth_rust.groupby("Type") if obj_type != "Type"}
eth_java_dfs = {obj_type+java_text: sub_df.reset_index() for obj_type, sub_df in eth_java.groupby("Type") if obj_type != "Type"}
# ib_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib.groupby("Type") if obj_type != "Type"}
# ib_java_dfs = {obj_type+ib_text: sub_df for obj_type, sub_df in ib_java.groupby("Type") if obj_type != "Type"}

#%%
cols = ["NClients","Latency","Throughput"]
# eth_rust_groups = eth_rust.groupby(["Type","NClients"])
grouped_java = eth_java.groupby(["Type"])[cols]

fig_lat, ax_lat = plt.subplots(figsize=(8, 5))
fig_thr, ax_thr = plt.subplots(figsize=(8, 5))

ax = [ax_lat,ax_thr]
alpha=0.2

for t, sub in eth_rust.groupby(["Type"])[cols]:
    sub = sub.set_index("NClients")
    num_clients = sub.index.values

    for c,a in zip(sub.columns,ax):
        a.plot(num_clients,sub[c],label=t,marker='.')
        a.fill_between(num_clients,sub[c],sub[c],alpha=alpha)


for t, sub in grouped_java:
    sub = sub.sort_values('NClients')
    print(sub)
    mean = sub.groupby(['NClients']).mean()
    std = sub.groupby(['NClients']).std()
    num_clients = mean.index.values
    lab = t[0]+java_text
    for c,a in zip(mean.columns,ax):
        m = mean[c]
        s = std[c]

        a.plot(num_clients,m,label=lab,marker='.')
        a.fill_between(num_clients,m-s,m+s,alpha=alpha)

        a.set_xlabel('Number of Clients')
        a.legend()
        a.grid(True, alpha=0.3)
        a.set_yscale("log")

ax_lat.set_ylabel('Average Latency (μsec)')
ax_lat.set_title('Average Latency vs Number of Clients')

ax_thr.set_ylabel('Average Throughput (bps)')
ax_thr.set_title('Average Throughput vs Number of Clients')

plt.tight_layout()
plt.savefig('latency_plot.png', dpi=150)
plt.show()

# %%
