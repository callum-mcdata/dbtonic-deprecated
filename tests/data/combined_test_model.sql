with staging as (

    select 
        staging_id,
        staging_content,
        staging_date
    from {{ ref('stg_model_name' )}}

)

select * from staging