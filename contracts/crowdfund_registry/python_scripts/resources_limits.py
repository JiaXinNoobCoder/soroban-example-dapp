import pandas as pd

# Specify the file paths
v1_path = '/home/jiaxin/projects/stellar/dapp_crowdfunding/soroban-example-dapp/contracts/crowdfund_registry/creat_batch_crowdfunds.txt'
v2_path = '/home/jiaxin/projects/stellar/dapp_crowdfunding/soroban-example-dapp/contracts/crowdfund_registry/creat_batch_crowdfunds_v2.txt'

# Read the .txt files, specifying that the header is on the 3rd line (index 2) and using whitespace as the delimiter
df_v1 = pd.read_csv(v1_path, skiprows=2, delim_whitespace=True)
df_v2 = pd.read_csv(v2_path, skiprows=2, delim_whitespace=True)

# Convert relevant columns to numeric types
headers = df_v1.columns[1:]  # Assuming the first column is not numeric
cost_types = df_v1[df_v1.columns[0]]
df_v1[headers] = df_v1[headers].apply(pd.to_numeric)
df_v2[headers] = df_v2[headers].apply(pd.to_numeric)
difference_df = df_v1[headers] - df_v2[headers]
difference_df = difference_df.round(2)

print(headers)
difference_df.insert(0, 'cost_types', cost_types)
print(difference_df)
# Compute the variation rate and convert to percentage
#variation_rate = ((df_v1 - df_v2) / df_v1) * 100

# Round the variation rate to 2 decimal places
#variation_rate = variation_rate.round(2)

# Display the variation rate as percentage
#print(variation_rate)