package problem

//Problem represents the interface of problem
type Problem interface {
	GetContestName()
	GetProblemName() (string, error)
	GetTimeLimit()
	GetMemoryLimit()
	GetSampleInputs()
	GetSampleOutpus()
}
