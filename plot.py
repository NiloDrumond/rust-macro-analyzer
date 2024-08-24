import json
import matplotlib.pyplot as plt

# Step 1: Read JSON data from a file
with open('data/analyzis.json', 'r') as file:
    data = json.load(file)

# Step 2: Extract the total macro count for each repo
repo_macro_counts = {}
for repo_name, repo_data in data['repos'].items():
    if repo_data.get('macro_usage'):
        macro_invocation_count = repo_data['macro_usage'].get('macro_invocation_count', 0)
        attribute_macro_invocation_count = repo_data['macro_usage'].get('attribute_macro_invocation_count', 0)
        derive_macro_count = repo_data['macro_usage']['derive_macro_usage'].get('count', 0)
        total_macro_count = macro_invocation_count + attribute_macro_invocation_count + derive_macro_count
        repo_macro_counts[repo_name] = total_macro_count

# Step 3: Sort the repos by total macro count in descending order
sorted_repos = sorted(repo_macro_counts.items(), key=lambda x: x[1], reverse=True)
top_20_repos = sorted_repos[:20]
repo_names = [repo[0] for repo in top_20_repos]
macro_counts = [repo[1] for repo in top_20_repos]
# repo_names = [repo[0] for repo in sorted_repos]
# macro_counts = [repo[1] for repo in sorted_repos]

# Step 4: Create a bar graph
plt.figure(figsize=(10, 6))
plt.bar(repo_names, macro_counts, color='skyblue')
plt.title('Total Macro Count per Repo')
plt.xlabel('Repo')
plt.ylabel('Total Macro Count')
plt.xticks(rotation=45, ha='right')
plt.tight_layout()


# Display the plot
plt.savefig('macro_counts.png')
