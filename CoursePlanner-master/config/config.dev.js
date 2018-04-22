module.exports = {
    db: {
        url: 'mongodb://mongo/' + (process.env.TEST ? 'test' : 'course-planner')
    },

    //~tr:JWTSecret
    jwt: {
        secret: process.env.JWT_SECRET
    },

    google: {
        clientSecret: process.env.GOOGLE_CLIENT_SECRET
    }
};
