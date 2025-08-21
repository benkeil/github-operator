# To Discuss

## Decisions

- Every api touched by this operator is the single source of truth and overrides manually changes

## Considerations

### For Developer

- multiple Resources (at least for the GitHub API) makes it easier to manage single elements of a list without a single
  source of truth
- multiple Resources makes it easier to save the state

### For User

- multiple Resources makes it probably harder to write manifests (if you stick to yaml without any templating)

## GitHub API

### Auto Links

<https://docs.github.com/en/rest/repos/autolinks?apiVersion=2022-11-28>

- Supports only `Create`
- Primary Key is `key_prefix`
- Resource ID is `id: Int`

> Manage all Auto Links? ✅

> Dedicated Resource? ❌

### Team Permissions

<https://docs.github.com/en/rest/teams/teams?apiVersion=2022-11-28#add-or-update-team-repository-permissions>

- Supports `Upsert`
- We can't manage everything, because the organization adds team permissions

> Manage all Auto Links? ✅

> Add a dedicated field `ownerTeam` to spec? ❌ -> the operator should run as admin and has always permissions to give your team permissions back

> Dedicated Resource? ❌

### Collaborators

<https://docs.github.com/en/rest/collaborators/collaborators?apiVersion=2022-11-28#add-a-repository-collaborator>

- Supports `Upsert`
- Returns `204` if we update an existing collaborator
- We can't manage everything, because the organization could add user permissions

> Manage all Auto Links? ✅

> Dedicated Resource? ❌

### Automated Security Fixes

- Easy nothing to discuss

### Rule Sets

<https://docs.github.com/en/rest/repos/rules?apiVersion=2022-11-28>

- Supports `Create` and `Update`
- Resource ID is `id: Int`

> Dedicated Resource? ❌

> Manage all Rule Sets? ✅
