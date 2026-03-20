mod common;
use common::transform;
use mud_transformer::output::{
    EntityFragment, MapperFragment, MxpFragment, OutputFragment, VariableFragment,
};

#[test]
fn set_variables() {
    let output =
        transform("\x1B[1z<!EL PR Flag=Prompt><!EL Hp Flag='Set HP'><!EL RNum Flag='RoomNum' EMPTY>Before<PR>This is <Var QE PUBLISH><Hp>hp1</Hp></Var><Hp>hp2</Hp></PR>After<RNum 107>")
            .output()
            .into_iter()
            .filter_map(|frag| match frag { OutputFragment::Mxp(frag) => Some(frag), _ => None })
            .collect::<Vec<_>>();

    assert_eq!(
        output,
        &[
            MxpFragment::Variable(VariableFragment {
                name: "HP".into(),
                value: "hp1".into(),
            }),
            MxpFragment::Entity(EntityFragment {
                name: "QE".into(),
                value: Some("hp1".into()),
                publish: true,
            }),
            MxpFragment::Variable(VariableFragment {
                name: "HP".into(),
                value: "hp2".into(),
            }),
            MxpFragment::Mapper(MapperFragment {
                parse_as: mxp::ParseAs::Prompt,
                value: "This is hp1hp2".into(),
            }),
            MxpFragment::Mapper(MapperFragment {
                parse_as: mxp::ParseAs::RoomNum,
                value: "107".into(),
            })
        ]
    );
}
