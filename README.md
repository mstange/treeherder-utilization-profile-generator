# treeherder-utilization-profile-generator

Example profile: https://share.firefox.dev/3ZNVOIf

Usage: `cargo run --release -- ~/Downloads/a55-utilization.csv`

Then load `~/Downloads/a55-utilization.csv-profile.json` in https://profiler.firefox.com/ .

Generate the CSV using sql.telemetry.mozilla.org from the Treeherder database like this:

```sql
SELECT *
FROM
  (SELECT start_time,
          task_id,
          repository.name AS repository_name,
          machine_platform.platform,
          job_group.symbol AS job_group_symbol,
          job_type.name AS job_type_name,
          EXTRACT(EPOCH
                  FROM (end_time - start_time))::integer AS duration,
          who
   FROM job
   JOIN taskcluster_metadata ON job.id = job_id
   JOIN job_group ON job.job_group_id = job_group.id
   JOIN machine ON job.machine_id = machine.id
   JOIN machine_platform ON job.machine_platform_id = machine_platform.id
   JOIN job_type ON job.job_type_id = job_type.id
   JOIN push ON job.push_id = push.id
   JOIN repository ON push.repository_id = repository.id
   WHERE end_time != TO_TIMESTAMP(0)
     AND DATE(start_time) > CURRENT_DATE - INTERVAL '1 month'
     AND machine.name LIKE 'a55%'
   LIMIT 100000) AS sub
ORDER BY start_time
```

Or here's one for macmini utilization:

```sql
SELECT *
FROM
  (SELECT start_time,
          task_id,
          repository.name AS repository_name,
          machine_platform.platform,
          job_group.symbol AS job_group_symbol,
          job_type.name AS job_type_name,
          EXTRACT(EPOCH
                  FROM (end_time - start_time))::integer AS duration,
          who
   FROM job
   JOIN taskcluster_metadata ON job.id = job_id
   JOIN job_group ON job.job_group_id = job_group.id
   JOIN machine ON job.machine_id = machine.id
   JOIN machine_platform ON job.machine_platform_id = machine_platform.id
   JOIN job_type ON job.job_type_id = job_type.id
   JOIN push ON job.push_id = push.id
   JOIN repository ON push.repository_id = repository.id
   WHERE end_time != TO_TIMESTAMP(0)
     AND DATE(start_time) > CURRENT_DATE - INTERVAL '7 days'
     AND machine.name LIKE 'macmini-r8-%'
   LIMIT 100000) AS sub
ORDER BY start_time
```
