SELECT *, fuzzy_score(name, $1) > 0.3 as score
FROM packages
-- WHERE score > 0.3
ORDER BY score ASC
LIMIT $2

