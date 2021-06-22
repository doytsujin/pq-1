use std::collections::HashSet;

use super::value::{ExprValue, ExprValueIter, ExprValueKind, InstantVector, RangeVector};
use crate::model::{
    labels::{LabelValue, LabelsTrait},
    types::SampleValue,
};
use crate::parser::ast::FunctionName;

pub(super) enum FuncCallArg {
    Number(f64),
    String(LabelValue),
    ValueIter(Box<dyn ExprValueIter>),
}

pub(super) fn create_func_call_executor(
    func_name: FunctionName,
    mut args: Vec<FuncCallArg>,
) -> Box<dyn ExprValueIter> {
    use FunctionName::*;

    match func_name {
        CountOverTime | MinOverTime | MaxOverTime | SumOverTime => {
            assert!(args.len() == 1);
            if let Some(FuncCallArg::ValueIter(inner)) = args.pop() {
                return Box::new(AggOverTimeFuncExecutor::new(func_name, inner));
            }
            panic!("unexpected argument type");
        }
        _ => unimplemented!("Coming soon..."),
    }
}

struct AggOverTimeFuncExecutor {
    func_name: FunctionName,
    inner: Box<dyn ExprValueIter>,
}

impl AggOverTimeFuncExecutor {
    fn new(func_name: FunctionName, inner: Box<dyn ExprValueIter>) -> Self {
        Self { func_name, inner }
    }

    fn next_count(&self, v: RangeVector) -> InstantVector {
        let samples = v
            .samples()
            .into_iter()
            .map(|(labels, values)| (labels.without(&HashSet::new()), values.len() as SampleValue))
            .collect();
        InstantVector::new(v.timestamp(), samples)
    }
}

impl std::iter::Iterator for AggOverTimeFuncExecutor {
    type Item = ExprValue;

    fn next(&mut self) -> Option<Self::Item> {
        use FunctionName::*;

        let v = match self.inner.next() {
            Some(ExprValue::RangeVector(v)) => v,
            None => return None,
            _ => unreachable!("bug"),
        };

        match self.func_name {
            CountOverTime => Some(ExprValue::InstantVector(self.next_count(v))),
            _ => unimplemented!(),
        }
    }
}

impl ExprValueIter for AggOverTimeFuncExecutor {
    fn value_kind(&self) -> ExprValueKind {
        ExprValueKind::InstantVector
    }
}
