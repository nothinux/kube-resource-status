# :crab: kube resource-status

kube resource-status is a tool that provide kubernetes cluster resource information, including cpu, memory, storage and number of pods. 

### Usage
```
$ kube-resource-status

╭────────────────────┬───────────────┬────────────────┬────────────────┬──────────╮
│ node_name          │ cpu           │ mem            │ storage        │ pods     │
├────────────────────┼───────────────┼────────────────┼────────────────┼──────────┤
│ control-plane      │ 950m (11.88%) │ 290Mi (2.43%)  │ 0Mi (0.00%)    │ 9 / 110  │
│ worker             │ 600m (7.50%)  │ 550Mi (4.61%)  │ 1000Mi (2.12%) │ 3 / 110  │
│ worker2            │ 600m (7.50%)  │ 550Mi (4.61%)  │ 1000Mi (2.12%) │ 3 / 110  │
│ *                  │ 2150m (8.96%) │ 1390Mi (3.88%) │ 2000Mi (1.41%) │ 15 / 330 │
╰────────────────────┴───────────────┴────────────────┴────────────────┴──────────╯
```
