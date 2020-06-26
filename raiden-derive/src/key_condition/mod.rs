// #partitionKeyName = :partitionkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName = :sortkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName < :sortkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName <= :sortkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName > :sortkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName >= :sortkeyval
// #partitionKeyName = :partitionkeyval AND #sortKeyName BETWEEN :sortkeyval1 AND :sortkeyval2
// #partitionKeyName = :partitionkeyval AND begins_with ( #sortKeyName, :sortkeyval )
pub mod builder;

pub use builder::*;
