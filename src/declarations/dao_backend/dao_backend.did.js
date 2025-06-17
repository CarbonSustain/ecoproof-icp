export const idlFactory = ({ IDL }) => {
  const Challenge = IDL.Record({
    'id' : IDL.Nat64,
    'latitude' : IDL.Float64,
    'title' : IDL.Text,
    'expiration' : IDL.Nat64,
    'longitude' : IDL.Float64,
    'picture_url' : IDL.Text,
    'radius_m' : IDL.Float64,
  });
  const PostStatus = IDL.Variant({
    'OPEN' : IDL.Null,
    'PAID' : IDL.Null,
    'EXPIRED' : IDL.Null,
    'PENDING' : IDL.Null,
  });
  const Role = IDL.Variant({
    'User' : IDL.Null,
    'Admin' : IDL.Null,
    'Moderator' : IDL.Null,
  });
  const VoteSummary = IDL.Record({
    'upvotes' : IDL.Nat32,
    'data_id' : IDL.Nat64,
    'downvotes' : IDL.Nat32,
  });
  const UserSubmission = IDL.Record({
    'status' : PostStatus,
    'data_id' : IDL.Nat64,
    'data' : IDL.Record({
      'latitude' : IDL.Float64,
      'submission_photo_url' : IDL.Text,
      'temperature' : IDL.Float64,
      'city' : IDL.Text,
      'longitude' : IDL.Float64,
      'timestamp' : IDL.Nat64,
      'weather' : IDL.Text,
    }),
    'user' : IDL.Text,
    'expiration_timestamp' : IDL.Nat64,
    'rewarded' : IDL.Bool,
  });
  const UserSubmissionSummary = IDL.Record({
    'status' : PostStatus,
    'data_id' : IDL.Nat64,
    'city' : IDL.Text,
  });
  const Vote = IDL.Record({
    'data_id' : IDL.Nat64,
    'user' : IDL.Text,
    'vote_value' : IDL.Bool,
    'submission_id' : IDL.Nat64,
  });
  return IDL.Service({
    'create_challenge' : IDL.Func(
        [IDL.Text, IDL.Float64, IDL.Float64, IDL.Float64, IDL.Nat64, IDL.Text],
        [IDL.Nat64],
        [],
      ),
    'create_tg_user' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text, IDL.Text, IDL.Text, IDL.Bool, IDL.Text],
        [IDL.Text],
        [],
      ),
    'delete_vote' : IDL.Func([IDL.Text, IDL.Nat64], [IDL.Text], []),
    'finalize_post_status' : IDL.Func([IDL.Nat64], [IDL.Text], []),
    'get_active_challenges' : IDL.Func(
        [IDL.Float64, IDL.Float64],
        [IDL.Vec(Challenge)],
        ['query'],
      ),
    'get_all_expiration_times' : IDL.Func(
        [],
        [
          IDL.Vec(
            IDL.Record({
              'data_id' : IDL.Nat64,
              'expiration_timestamp' : IDL.Nat64,
            })
          ),
        ],
        [],
      ),
    'get_all_submissions' : IDL.Func(
        [],
        [
          IDL.Vec(
            IDL.Record({
              'status' : PostStatus,
              'data_id' : IDL.Nat64,
              'data' : IDL.Record({
                'latitude' : IDL.Float64,
                'submission_photo_url' : IDL.Text,
                'temperature' : IDL.Float64,
                'city' : IDL.Text,
                'longitude' : IDL.Float64,
                'timestamp' : IDL.Nat64,
                'weather' : IDL.Text,
              }),
              'user' : IDL.Text,
              'rewarded' : IDL.Bool,
            })
          ),
        ],
        [],
      ),
    'get_all_users' : IDL.Func(
        [],
        [
          IDL.Vec(
            IDL.Record({
              'username' : IDL.Opt(IDL.Text),
              'balance' : IDL.Nat64,
              'language_code' : IDL.Opt(IDL.Text),
              'role' : Role,
              'wallet_address' : IDL.Opt(IDL.Text),
              'profile_picture_url' : IDL.Opt(IDL.Text),
              'user_id' : IDL.Text,
              'is_bot' : IDL.Bool,
              'first_name' : IDL.Opt(IDL.Text),
              'last_name' : IDL.Opt(IDL.Text),
            })
          ),
        ],
        [],
      ),
    'get_balance' : IDL.Func([IDL.Text], [IDL.Nat64], []),
    'get_challenge' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : Challenge, 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_challenges_by_radius' : IDL.Func(
        [IDL.Float64, IDL.Float64, IDL.Float64],
        [IDL.Vec(Challenge)],
        ['query'],
      ),
    'get_challenges_expiring_soon' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(Challenge)],
        ['query'],
      ),
    'get_expiration_time' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'get_leaderboard_by_total_votes' : IDL.Func(
        [],
        [IDL.Vec(VoteSummary)],
        ['query'],
      ),
    'get_leaderboard_by_upvotes' : IDL.Func(
        [],
        [IDL.Vec(VoteSummary)],
        ['query'],
      ),
    'get_post_status' : IDL.Func([IDL.Nat64], [IDL.Text], []),
    'get_rewarded_submissions' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(UserSubmission)],
        ['query'],
      ),
    'get_submission' : IDL.Func(
        [IDL.Nat64],
        [
          IDL.Variant({
            'Ok' : IDL.Record({
              'status' : PostStatus,
              'data_id' : IDL.Nat64,
              'data' : IDL.Record({
                'latitude' : IDL.Float64,
                'submission_photo_url' : IDL.Text,
                'temperature' : IDL.Float64,
                'city' : IDL.Text,
                'longitude' : IDL.Float64,
                'timestamp' : IDL.Nat64,
                'weather' : IDL.Text,
              }),
              'user' : IDL.Text,
              'rewarded' : IDL.Bool,
            }),
            'Err' : IDL.Text,
          }),
        ],
        [],
      ),
    'get_submissions_by_challenge' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(UserSubmission)],
        ['query'],
      ),
    'get_submissions_by_city' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(UserSubmission)],
        ['query'],
      ),
    'get_tg_user' : IDL.Func(
        [IDL.Text],
        [
          IDL.Variant({
            'Ok' : IDL.Record({
              'username' : IDL.Opt(IDL.Text),
              'balance' : IDL.Nat64,
              'language_code' : IDL.Opt(IDL.Text),
              'role' : Role,
              'wallet_address' : IDL.Opt(IDL.Text),
              'profile_picture_url' : IDL.Opt(IDL.Text),
              'user_id' : IDL.Text,
              'is_bot' : IDL.Bool,
              'first_name' : IDL.Opt(IDL.Text),
              'last_name' : IDL.Opt(IDL.Text),
            }),
            'Err' : IDL.Text,
          }),
        ],
        [],
      ),
    'get_user_posts' : IDL.Func(
        [IDL.Text],
        [
          IDL.Vec(
            IDL.Record({
              'status' : PostStatus,
              'data_id' : IDL.Nat64,
              'data' : IDL.Record({
                'latitude' : IDL.Float64,
                'submission_photo_url' : IDL.Text,
                'temperature' : IDL.Float64,
                'city' : IDL.Text,
                'longitude' : IDL.Float64,
                'timestamp' : IDL.Nat64,
                'weather' : IDL.Text,
              }),
              'user' : IDL.Text,
              'rewarded' : IDL.Bool,
            })
          ),
        ],
        [],
      ),
    'get_user_role' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : Role, 'Err' : IDL.Text })],
        [],
      ),
    'get_user_submission_locations' : IDL.Func(
        [IDL.Text],
        [
          IDL.Vec(
            IDL.Record({
              'status' : IDL.Variant({
                'OPEN' : IDL.Null,
                'PAID' : IDL.Null,
                'EXPIRED' : IDL.Null,
                'PENDING' : IDL.Null,
              }),
              'latitude' : IDL.Float64,
              'data_id' : IDL.Nat64,
              'longitude' : IDL.Float64,
            })
          ),
        ],
        ['query'],
      ),
    'get_user_submission_summary' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(UserSubmissionSummary)],
        ['query'],
      ),
    'get_user_submissions_by_challenge' : IDL.Func(
        [IDL.Text, IDL.Nat64],
        [IDL.Vec(UserSubmission)],
        ['query'],
      ),
    'get_vote_summary' : IDL.Func(
        [IDL.Nat64],
        [
          IDL.Record({
            'upvotes' : IDL.Nat32,
            'data_id' : IDL.Nat64,
            'downvotes' : IDL.Nat32,
          }),
        ],
        [],
      ),
    'get_votes_by_user' : IDL.Func([IDL.Text], [IDL.Vec(Vote)], ['query']),
    'mark_submission_rewarded' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'reward_user' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'submit_weather_data' : IDL.Func(
        [
          IDL.Text,
          IDL.Float64,
          IDL.Float64,
          IDL.Text,
          IDL.Float64,
          IDL.Text,
          IDL.Text,
        ],
        [IDL.Nat64],
        [],
      ),
    'submit_weather_data_with_challenge' : IDL.Func(
        [
          IDL.Text,
          IDL.Float64,
          IDL.Float64,
          IDL.Text,
          IDL.Float64,
          IDL.Text,
          IDL.Text,
          IDL.Nat64,
        ],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'update_user_role' : IDL.Func(
        [IDL.Text, IDL.Text, Role],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'update_vote' : IDL.Func([IDL.Text, IDL.Nat64, IDL.Bool], [IDL.Text], []),
    'update_wallet_address' : IDL.Func([IDL.Text, IDL.Text], [IDL.Text], []),
    'vote_on_data' : IDL.Func([IDL.Text, IDL.Nat64, IDL.Bool], [IDL.Text], []),
  });
};
export const init = ({ IDL }) => { return []; };
