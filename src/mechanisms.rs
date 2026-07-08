use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn all_or_nothing<T, F, FUT>(
    func: F,
    timeout_ms: usize,
) -> Result<T, Box<dyn Error>>
where
    FUT: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    F: Fn() -> FUT,
{
    let mut res = func().await;
    let timenow = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    while {
        res.is_err()
            && timeout_ms != 0usize
            && timeout_ms
                > (SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() - timenow) as usize
    } {
        res = func().await;
    }
    res
}

pub async fn one_time<T, O, F, K, FUT>(
    func: F,
    func_key: for<'a> fn(&'a O) -> Result<&'a K, Box<dyn Error>>,
    func_first_key: for<'a> fn(&'a T) -> Result<&'a K, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>>
where
    for<'c> &'c T: IntoIterator<Item = &'c O>,
    for<'b> &'b K: PartialEq,
    F: Fn() -> FUT,
    FUT: Future<Output = Result<T, Box<dyn Error>>>,
{
    let mut res = func().await?;
    let mut first = func_first_key(&res)?;
    while res.into_iter().any(|v| {
        let key = func_key(v);
        match key {
            Ok(k) => k != first,
            Err(_) => false,
        }
    }) {
        res = func().await?;
        first = func_first_key(&res)?;
    }
    Ok(res)
}

pub async fn one_time_hm<T, F, C, K, V, FUT>(
    func: F,
    func_key: for<'d> fn(&'d (&'d K, &'d V)) -> Result<&'d C, Box<dyn Error>>,
    func_first_key: for<'a> fn(&'a T) -> Result<&'a C, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>>
where
    for<'c> &'c T: IntoIterator<Item = (&'c K, &'c V)>,
    for<'b> &'b C: PartialEq,
    F: Fn() -> FUT,
    FUT: Future<Output = Result<T, Box<dyn Error>>>,
{
    let mut res = func().await?;
    let mut first = func_first_key(&res)?;
    while res.into_iter().any(|v| {
        let key = func_key(&v);
        match key {
            Ok(k) => k != first,
            Err(_) => false,
        }
    }) {
        res = func().await?;
        first = func_first_key(&res)?;
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::hash_map::RandomState;
    use std::{collections::HashMap, error::Error};

    use tokio;

    #[tokio::test]
    async fn one_time_res_1() -> Result<(), Box<dyn Error>> {
        one_time(
            async || Ok(vec![1, 1]),
            |v| Ok(v),
            |v| Ok(v.first().ok_or(Box::<dyn Error>::from("first el err"))?),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn one_time_res_2() -> Result<(), Box<dyn Error>> {
        one_time_hm(
            async || {
                Ok(HashMap::<_, _, RandomState>::from_iter([
                    ("1", "word"),
                    ("2", "word"),
                ]))
            },
            |v| Ok(v.1),
            |v| {
                Ok(v.iter()
                    .next()
                    .ok_or(Box::<dyn Error>::from("first el err"))?
                    .1)
            },
        )
        .await?;
        Ok(())
    }
}
