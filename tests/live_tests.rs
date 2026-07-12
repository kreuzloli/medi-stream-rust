use medi_stream_rust::live::live_model::{
    FileObject, LiveRoom, LiveRoomDetail, LiveRoomStream, SaveFileObjectReq, SaveLiveRoomReq,
    SaveLiveRoomStreamReq,
};
use medi_stream_rust::live::live_service::{
    build_live_room_detail, validate_save_file_object_req, validate_save_live_room_req,
    validate_save_live_room_stream_req,
};

/// 验证保存请求的核心行为。
#[test]
fn save_file_object_requires_name_and_url() {
    let req = SaveFileObjectReq {
        file_name: " ".to_string(),
        file_url: " ".to_string(),
        mime_type: None,
        file_size: None,
        sha256: None,
    };

    let err = validate_save_file_object_req(&req).expect_err("blank file fields must be rejected");

    assert!(err.to_string().contains("文件名称不能为空"));
}

/// 验证保存请求的核心行为。
#[test]
fn save_live_room_accepts_status_two_for_banned_room() {
    let req = SaveLiveRoomReq {
        status: Some(2),
        ..valid_room_req()
    };

    validate_save_live_room_req(&req).expect("room status 2 should be accepted");
}

#[test]
fn save_live_room_requires_exactly_one_owner() {
    let no_owner = SaveLiveRoomReq {
        owner_user_id: None,
        owner_admin_id: None,
        ..valid_room_req()
    };
    let both_owners = SaveLiveRoomReq {
        owner_user_id: Some(1),
        owner_admin_id: Some(2),
        ..valid_room_req()
    };

    let no_owner_err =
        validate_save_live_room_req(&no_owner).expect_err("room without owner must be rejected");
    let both_owners_err = validate_save_live_room_req(&both_owners)
        .expect_err("room with user and administrator owners must be rejected");

    assert!(no_owner_err.to_string().contains("房主必须且只能指定一个"));
    assert!(both_owners_err
        .to_string()
        .contains("房主必须且只能指定一个"));
}

#[test]
fn save_live_room_treats_zero_id_as_a_supplied_owner() {
    let req = SaveLiveRoomReq {
        owner_user_id: Some(0),
        owner_admin_id: Some(2),
        ..valid_room_req()
    };

    let err = validate_save_live_room_req(&req)
        .expect_err("two supplied owner fields must be rejected before reference lookup");

    assert!(err.to_string().contains("房主必须且只能指定一个"));
}

#[test]
fn save_live_room_accepts_administrator_owner() {
    let req = SaveLiveRoomReq {
        owner_user_id: None,
        owner_admin_id: Some(2),
        ..valid_room_req()
    };

    validate_save_live_room_req(&req).expect("administrator owner should be accepted");
}

#[test]
fn save_live_room_rejects_invalid_top_flag() {
    let req = SaveLiveRoomReq {
        is_top: Some(2),
        ..valid_room_req()
    };

    let err = validate_save_live_room_req(&req).expect_err("invalid top flag must be rejected");

    assert!(err.to_string().contains("置顶标记只能是0或1"));
}

/// 验证保存请求的核心行为。
#[test]
fn save_live_stream_rejects_invalid_default_flag() {
    let req = SaveLiveRoomStreamReq {
        is_default: Some(2),
        ..valid_stream_req()
    };

    let err = validate_save_live_room_stream_req(&req).expect_err("invalid default flag rejected");

    assert!(err.to_string().contains("默认流标记只能是0或1"));
}

/// 验证直播业务模型的核心行为。
#[test]
fn live_room_detail_can_contain_multiple_streams() {
    let room = LiveRoom {
        id: 10,
        owner_user_id: Some(1),
        owner_admin_id: None,
        room_code: "room001".to_string(),
        title: "示教直播间".to_string(),
        description: None,
        cover_file_id: None,
        department_id: Some(1),
        disease_id: Some(2),
        is_top: 1,
        status: 1,
        is_deleted: 0,
        created_at: None,
        updated_at: None,
    };
    let streams = vec![
        LiveRoomStream {
            id: 100,
            room_id: 10,
            stream_code: "main".to_string(),
            stream_name: "room001-main".to_string(),
            title: Some("主画面".to_string()),
            sort_no: 0,
            is_default: 1,
            status: 1,
            is_deleted: 0,
            created_at: None,
            updated_at: None,
        },
        LiveRoomStream {
            id: 101,
            room_id: 10,
            stream_code: "side".to_string(),
            stream_name: "room001-side".to_string(),
            title: Some("侧画面".to_string()),
            sort_no: 1,
            is_default: 0,
            status: 1,
            is_deleted: 0,
            created_at: None,
            updated_at: None,
        },
    ];

    let detail: LiveRoomDetail = build_live_room_detail(room, streams);

    assert_eq!(detail.streams.len(), 2);
    assert_eq!(detail.streams[0].stream_code, "main");
    assert_eq!(detail.streams[1].stream_code, "side");
}

/// 构造测试使用的有效请求对象。
fn valid_room_req() -> SaveLiveRoomReq {
    SaveLiveRoomReq {
        owner_user_id: Some(1),
        owner_admin_id: None,
        room_code: "room001".to_string(),
        title: "示教直播间".to_string(),
        description: Some("多路直播".to_string()),
        cover_file_id: Some(1),
        department_id: None,
        disease_id: None,
        is_top: Some(0),
        status: Some(1),
    }
}

/// 构造测试使用的有效请求对象。
fn valid_stream_req() -> SaveLiveRoomStreamReq {
    SaveLiveRoomStreamReq {
        room_id: 10,
        stream_code: "main".to_string(),
        stream_name: "room001-main".to_string(),
        title: Some("主画面".to_string()),
        sort_no: Some(0),
        is_default: Some(1),
        status: Some(1),
    }
}

/// 构造示例数据，帮助测试代码表达字段含义。
#[allow(dead_code)]
fn _example_file_object() -> FileObject {
    FileObject {
        id: 1,
        file_name: "cover.png".to_string(),
        file_url: "https://example.com/cover.png".to_string(),
        mime_type: Some("image/png".to_string()),
        file_size: Some(1024),
        sha256: None,
        created_at: None,
    }
}
