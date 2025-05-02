use super::*;

#[test]
fn test_invalid_args() {
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::Linear,
        Extrapolate::Error,
    )
    .unwrap();
    assert!(matches!(
        interp.interpolate(&[]).unwrap_err(),
        InterpolateError::PointLength(_)
    ));
    assert_eq!(interp.interpolate(&[1.0]).unwrap(), 0.4);
}

#[test]
fn test_linear() {
    let x = array![0., 1., 2., 3., 4.];
    let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
    let interp = Interp1D::new(x.view(), f_x.view(), strategy::Linear, Extrapolate::Error).unwrap();
    // Check that interpolating at grid points just retrieves the value
    for (i, x_i) in x.iter().enumerate() {
        assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
    }
    assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.95);
    assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
}

#[test]
fn test_left_nearest() {
    let x = array![0., 1., 2., 3., 4.];
    let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
    let interp = Interp1D::new(
        x.view(),
        f_x.view(),
        strategy::LeftNearest,
        Extrapolate::Error,
    )
    .unwrap();
    // Check that interpolating at grid points just retrieves the value
    for (i, x_i) in x.iter().enumerate() {
        assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
    }
    assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[3.75]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
}

#[test]
fn test_right_nearest() {
    let x = array![0., 1., 2., 3., 4.];
    let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
    let interp = Interp1D::new(
        x.view(),
        f_x.view(),
        strategy::RightNearest,
        Extrapolate::Error,
    )
    .unwrap();
    // Check that interpolating at grid points just retrieves the value
    for (i, x_i) in x.iter().enumerate() {
        assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
    }
    assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[3.25]).unwrap(), 1.0);
    assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
}

#[test]
fn test_nearest() {
    let x = array![0., 1., 2., 3., 4.];
    let f_x = array![0.2, 0.4, 0.6, 0.8, 1.0];
    let interp =
        Interp1D::new(x.view(), f_x.view(), strategy::Nearest, Extrapolate::Error).unwrap();
    // Check that interpolating at grid points just retrieves the value
    for (i, x_i) in x.iter().enumerate() {
        assert_eq!(interp.interpolate(&[*x_i]).unwrap(), f_x[i]);
    }
    assert_eq!(interp.interpolate(&[3.00]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[3.25]).unwrap(), 0.8);
    assert_eq!(interp.interpolate(&[3.50]).unwrap(), 1.0);
    assert_eq!(interp.interpolate(&[3.75]).unwrap(), 1.0);
    assert_eq!(interp.interpolate(&[4.00]).unwrap(), 1.0);
}

#[test]
fn test_extrapolate_inputs() {
    // Incorrect extrapolation selection
    assert!(matches!(
        Interp1D::new(
            array![0., 1., 2., 3., 4.],
            array![0.2, 0.4, 0.6, 0.8, 1.0],
            strategy::Nearest,
            Extrapolate::Enable,
        )
        .unwrap_err(),
        ValidateError::ExtrapolateSelection(_)
    ));

    // Extrapolate::Error
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::Linear,
        Extrapolate::Error,
    )
    .unwrap();
    // Fail to extrapolate below lowest grid value
    assert!(matches!(
        interp.interpolate(&[-1.]).unwrap_err(),
        InterpolateError::ExtrapolateError(_)
    ));
    // Fail to extrapolate above highest grid value
    assert!(matches!(
        interp.interpolate(&[5.]).unwrap_err(),
        InterpolateError::ExtrapolateError(_)
    ));
}

#[test]
fn test_extrapolate_fill() {
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::Linear,
        Extrapolate::Fill(f64::NAN),
    )
    .unwrap();
    assert_eq!(interp.interpolate(&[1.5]).unwrap(), 0.5);
    assert_eq!(interp.interpolate(&[2.]).unwrap(), 0.6);
    assert!(interp.interpolate(&[-1.]).unwrap().is_nan());
    assert!(interp.interpolate(&[5.]).unwrap().is_nan());
}

#[test]
fn test_extrapolate_clamp() {
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::Linear,
        Extrapolate::Clamp,
    )
    .unwrap();
    assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.2);
    assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.0);
}

#[test]
fn test_extrapolate() {
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::Linear,
        Extrapolate::Enable,
    )
    .unwrap();
    assert_eq!(interp.interpolate(&[-1.]).unwrap(), 0.0);
    assert_approx_eq!(interp.interpolate(&[-0.75]).unwrap(), 0.05);
    assert_eq!(interp.interpolate(&[5.]).unwrap(), 1.2);
}

#[test]
fn test_partialeq() {
    #[derive(PartialEq)]
    #[allow(unused)]
    struct MyStruct(InterpData1DOwned<f64>);

    #[derive(PartialEq)]
    #[allow(unused)]
    struct MyStruct2(Interp1DOwned<f64, strategy::Linear>);
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let interp = Interp1D::new(
        array![0., 1., 2., 3., 4.],
        array![0.2, 0.4, 0.6, 0.8, 1.0],
        strategy::LeftNearest,
        Extrapolate::Error,
    )
    .unwrap();

    let ser = serde_json::to_string(&interp).unwrap();
    let de: Interp1DOwned<f64, strategy::LeftNearest> = serde_json::from_str(&ser).unwrap();
    assert_eq!(interp, de);
}
