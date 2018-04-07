var mongoose = require('mongoose');
var Schema = mongoose.Schema;

module.exports = mongoose.model('Plan', {
    title : {
        type: String,
        required: true
    },
    details : {
        type: String
    },
    tags : {
        type: [Schema.Types.Mixed]
    },
    years  : {
        type: Object,
    },
    colorscheme  : {
        type: Object,
    },
    public: {
        type: Boolean,
    },
    school: { 
        type: Schema.Types.ObjectId, 
        ref: 'School'
    },
    user  : { 
        type: Schema.Types.ObjectId, 
        ref: 'User',
        required: true
    }
});
