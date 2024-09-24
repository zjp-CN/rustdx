WITH 
  df AS (
  SELECT
    code,
  arrayLast(
      x->true, 
      arraySort(x->x.1, groupArray((
        date, close, factor
      )))
    ) AS t
  FROM
    rustdx.factor
  GROUP BY
    code
  )
SELECT code, t.1 AS date, t.2 AS close, t.3 AS factor FROM df;
