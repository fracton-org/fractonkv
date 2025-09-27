# 1️⃣ Create a list of allowed filenames from your enum
cat << 'EOF' | tr '[:upper:]' '[:lower:]' | sed 's/_/-/g' | sed 's/$/.json/' > keep_json.txt
Get
Set
SetNx
GetSet
Append
StrLen
MGet
MSet
Incr
Decr
IncrBy
DecrBy
HSet
HSetNx
HGet
HDel
HExists
HGetAll
HKeys
HVals
HLen
HIncrBy
HIncrByFloat
HMGet
HMSet
LPush
RPush
LPushX
RPushX
LPop
RPop
LLen
LIndex
LInsert
LRange
LRem
LTrim
RPopLPush
SAdd
SRem
SMembers
SCard
SIsMember
SPop
SRandMember
SMove
SDiff
SInter
SUnion
SDiffStore
SInterStore
SUnionStore
ZAdd
ZRem
ZCard
ZScore
ZRank
ZRevRank
ZRange
ZRevRange
ZRangeByScore
ZRevRangeByScore
ZRemRangeByRank
ZRemRangeByScore
ZIncrBy
ZCount
ZInterStore
ZUnionStore
EOF


# 2️⃣ Delete all JSON files except these
for f in commands/*.json; do
    grep -qxF "$(basename "$f")" keep_json.txt || rm "$f"
done

